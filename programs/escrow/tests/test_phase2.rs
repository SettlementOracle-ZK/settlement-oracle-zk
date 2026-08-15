use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, AccountSerialize, InstructionData, ToAccountMetas,
    },
    escrow::state::{EscrowAccount, EscrowStatus},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const POLICY_ID: [u8; 32] = [1u8; 32];
const ASSET_CLASS: [u8; 32] = *b"agriculture_climate\0\0\0\0\0\0\0\0\0\0\0\0\0";
const TRIGGER_THRESHOLD: i64 = 100_000_000_000;
const DEPOSIT_LAMPORTS: u64 = 500_000_000;
const POLICY_EXPIRY: i64 = 4_102_444_800;
const HOLDER_AIRDROP: u64 = 1_000_000;

fn setup_svm() -> (LiteSVM, Keypair, Pubkey) {
    let program_id = escrow::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/deploy/escrow.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 2_000_000_000).unwrap();
    (svm, payer, program_id)
}

fn send_ix(svm: &mut LiteSVM, payer: &Keypair, instruction: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "transaction failed: {:?}", res.err());
}

fn send_ix_expect_err(svm: &mut LiteSVM, payer: &Keypair, instruction: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    assert!(svm.send_transaction(tx).is_err());
}

fn policy_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[escrow::constants::POLICY_SEED, POLICY_ID.as_ref()], program_id)
}

fn escrow_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[escrow::constants::ESCROW_SEED, POLICY_ID.as_ref()], program_id)
}

fn load_escrow(svm: &LiteSVM, escrow: &Pubkey) -> EscrowAccount {
    let account = svm.get_account(escrow).unwrap();
    let mut data: &[u8] = &account.data;
    EscrowAccount::try_deserialize(&mut data).unwrap()
}

fn mark_triggered(svm: &mut LiteSVM, escrow: &Pubkey) {
    let mut acc = svm.get_account(escrow).expect("escrow account");
    let mut data: &[u8] = &acc.data;
    let mut state = EscrowAccount::try_deserialize(&mut data).unwrap();
    state.status = EscrowStatus::Triggered;
    let mut buf = Vec::with_capacity(acc.data.len());
    state.try_serialize(&mut buf).unwrap();
    acc.data = buf;
    svm.set_account(*escrow, acc).unwrap();
}

fn fund_policy_and_escrow(
    svm: &mut LiteSVM,
    payer: &Keypair,
    program_id: &Pubkey,
    holder: &Pubkey,
) -> (Pubkey, Pubkey) {
    let (policy, _) = policy_pda(program_id);
    let (escrow, _) = escrow_pda(program_id);

    send_ix(
        svm,
        payer,
        Instruction::new_with_bytes(
            *program_id,
            &escrow::instruction::InitializePolicy {
                policy_id: POLICY_ID,
                holder: *holder,
                expiry: POLICY_EXPIRY,
                asset_class: ASSET_CLASS,
            }
            .data(),
            escrow::accounts::InitializePolicy {
                authority: payer.pubkey(),
                policy,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    send_ix(
        svm,
        payer,
        Instruction::new_with_bytes(
            *program_id,
            &escrow::instruction::InitializeEscrow {
                policy_id: POLICY_ID,
                trigger_threshold: TRIGGER_THRESHOLD,
            }
            .data(),
            escrow::accounts::InitializeEscrow {
                authority: payer.pubkey(),
                policy,
                escrow,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    send_ix(
        svm,
        payer,
        Instruction::new_with_bytes(
            *program_id,
            &escrow::instruction::DepositPremium {
                amount: DEPOSIT_LAMPORTS,
            }
            .data(),
            escrow::accounts::DepositPremium {
                authority: payer.pubkey(),
                escrow,
                system_program: system_program::ID,
            }
            .to_account_metas(None),
        ),
    );

    (policy, escrow)
}

fn payout_ix(program_id: Pubkey, policy: Pubkey, escrow: Pubkey, holder: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &escrow::instruction::ExecutePayout {}.data(),
        escrow::accounts::ExecutePayout {
            escrow,
            policy,
            holder,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn pause_ix(program_id: Pubkey, authority: Pubkey, escrow: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &escrow::instruction::Pause {}.data(),
        escrow::accounts::PauseEscrow {
            authority,
            escrow,
        }
        .to_account_metas(None),
    )
}

#[test]
fn test_execute_payout_happy_path() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    svm.airdrop(&holder.pubkey(), HOLDER_AIRDROP).unwrap();
    let holder_before = svm.get_account(&holder.pubkey()).unwrap().lamports;

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    mark_triggered(&mut svm, &escrow);

    send_ix(
        &mut svm,
        &payer,
        payout_ix(program_id, policy, escrow, holder.pubkey()),
    );

    let escrow_state = load_escrow(&svm, &escrow);
    assert_eq!(escrow_state.status, EscrowStatus::Paid);
    assert_eq!(escrow_state.amount, 0);
    assert!(!escrow_state.paused);

    let holder_after = svm.get_account(&holder.pubkey()).unwrap().lamports;
    assert_eq!(holder_after, holder_before + DEPOSIT_LAMPORTS);
}

#[test]
fn test_pause_blocks_payout() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    svm.airdrop(&holder.pubkey(), HOLDER_AIRDROP).unwrap();

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    mark_triggered(&mut svm, &escrow);

    send_ix(
        &mut svm,
        &payer,
        pause_ix(program_id, payer.pubkey(), escrow),
    );
    assert!(load_escrow(&svm, &escrow).paused);

    send_ix_expect_err(
        &mut svm,
        &payer,
        payout_ix(program_id, policy, escrow, holder.pubkey()),
    );

    let escrow_state = load_escrow(&svm, &escrow);
    assert_eq!(escrow_state.status, EscrowStatus::Triggered);
    assert_eq!(escrow_state.amount, DEPOSIT_LAMPORTS);
}

#[test]
fn test_payout_without_trigger_fails() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    svm.airdrop(&holder.pubkey(), HOLDER_AIRDROP).unwrap();

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    assert_eq!(load_escrow(&svm, &escrow).status, EscrowStatus::Active);

    send_ix_expect_err(
        &mut svm,
        &payer,
        payout_ix(program_id, policy, escrow, holder.pubkey()),
    );
}

#[test]
fn test_double_payout_fails() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    svm.airdrop(&holder.pubkey(), HOLDER_AIRDROP).unwrap();

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    mark_triggered(&mut svm, &escrow);

    send_ix(
        &mut svm,
        &payer,
        payout_ix(program_id, policy, escrow, holder.pubkey()),
    );
    send_ix_expect_err(
        &mut svm,
        &payer,
        payout_ix(program_id, policy, escrow, holder.pubkey()),
    );
    assert_eq!(load_escrow(&svm, &escrow).status, EscrowStatus::Paid);
}

#[test]
fn test_unpause_allows_payout() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    svm.airdrop(&holder.pubkey(), HOLDER_AIRDROP).unwrap();

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    mark_triggered(&mut svm, &escrow);

    send_ix(
        &mut svm,
        &payer,
        pause_ix(program_id, payer.pubkey(), escrow),
    );
    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &escrow::instruction::Unpause {}.data(),
            escrow::accounts::PauseEscrow {
                authority: payer.pubkey(),
                escrow,
            }
            .to_account_metas(None),
        ),
    );
    assert!(!load_escrow(&svm, &escrow).paused);

    send_ix(
        &mut svm,
        &payer,
        payout_ix(program_id, policy, escrow, holder.pubkey()),
    );
    assert_eq!(load_escrow(&svm, &escrow).status, EscrowStatus::Paid);
}
