use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{clock::Clock, instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    escrow::state::{EscrowAccount, EscrowStatus, PolicyAccount},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const POLICY_ID: [u8; 32] = [1u8; 32];
const ASSET_CLASS: [u8; 32] = *b"agriculture_climate\0\0\0\0\0\0\0\0\0\0\0\0\0";
const TRIGGER_THRESHOLD: i64 = 100_000_000_000; // $100 with expo -8
const DEPOSIT_LAMPORTS: u64 = 500_000_000;
const POLICY_EXPIRY: i64 = 4_102_444_800; // 2099-12-31

fn setup_svm() -> (LiteSVM, Keypair, Pubkey) {
    let program_id = escrow::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    // Always resolve from workspace root: programs/escrow/../../target/deploy/
    // (CARGO_TARGET_TMPDIR varies if tests run from another package directory)
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

fn policy_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[escrow::constants::POLICY_SEED, POLICY_ID.as_ref()], program_id)
}

fn escrow_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[escrow::constants::ESCROW_SEED, POLICY_ID.as_ref()], program_id)
}

#[test]
fn test_initialize_policy_and_escrow_account_state() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    let (policy, _) = policy_pda(&program_id);
    let (escrow, _) = escrow_pda(&program_id);

    let init_policy_ix = Instruction::new_with_bytes(
        program_id,
        &escrow::instruction::InitializePolicy {
            policy_id: POLICY_ID,
            holder: holder.pubkey(),
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
    );
    send_ix(&mut svm, &payer, init_policy_ix);

    let policy_account = svm.get_account(&policy).unwrap();
    let mut policy_data: &[u8] = &policy_account.data;
    let policy_state = PolicyAccount::try_deserialize(&mut policy_data).unwrap();
    assert_eq!(policy_state.policy_id, POLICY_ID);
    assert_eq!(policy_state.holder, holder.pubkey());
    assert_eq!(policy_state.asset_class, ASSET_CLASS);

    let init_escrow_ix = Instruction::new_with_bytes(
        program_id,
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
    );
    send_ix(&mut svm, &payer, init_escrow_ix);

    let escrow_account = svm.get_account(&escrow).unwrap();
    let mut escrow_data: &[u8] = &escrow_account.data;
    let escrow_state = EscrowAccount::try_deserialize(&mut escrow_data).unwrap();
    assert_eq!(escrow_state.policy_id, POLICY_ID);
    assert_eq!(escrow_state.authority, payer.pubkey());
    assert_eq!(escrow_state.amount, 0);
    assert_eq!(escrow_state.trigger_threshold, TRIGGER_THRESHOLD);
    assert_eq!(escrow_state.status, EscrowStatus::Active);
}

#[test]
fn test_deposit_premium_updates_escrow_balance() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    let (policy, _) = policy_pda(&program_id);
    let (escrow, _) = escrow_pda(&program_id);

    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &escrow::instruction::InitializePolicy {
                policy_id: POLICY_ID,
                holder: holder.pubkey(),
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
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
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
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
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

    let escrow_account = svm.get_account(&escrow).unwrap();
    let mut escrow_data: &[u8] = &escrow_account.data;
    let escrow_state = EscrowAccount::try_deserialize(&mut escrow_data).unwrap();
    assert_eq!(escrow_state.amount, DEPOSIT_LAMPORTS);
    assert!(escrow_account.lamports >= DEPOSIT_LAMPORTS);
}

#[test]
fn test_deposit_premium_rejects_zero_amount() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    let (policy, _) = policy_pda(&program_id);
    let (escrow, _) = escrow_pda(&program_id);

    for ix in [
        Instruction::new_with_bytes(
            program_id,
            &escrow::instruction::InitializePolicy {
                policy_id: POLICY_ID,
                holder: holder.pubkey(),
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
        Instruction::new_with_bytes(
            program_id,
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
    ] {
        send_ix(&mut svm, &payer, ix);
    }

    let blockhash = svm.latest_blockhash();
    let deposit_ix = Instruction::new_with_bytes(
        program_id,
        &escrow::instruction::DepositPremium { amount: 0 }.data(),
        escrow::accounts::DepositPremium {
            authority: payer.pubkey(),
            escrow,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let msg = Message::new_with_blockhash(&[deposit_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(svm.send_transaction(tx).is_err());
}

fn send_ix_expect_err(svm: &mut LiteSVM, payer: &Keypair, instruction: Instruction) {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    assert!(svm.send_transaction(tx).is_err());
}

#[test]
fn test_initialize_policy_rejects_invalid_expiry() {
    let (mut svm, payer, program_id) = setup_svm();
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_700_000_000;
    svm.set_sysvar::<Clock>(&clock);

    let holder = Keypair::new();
    let (policy, _) = policy_pda(&program_id);
    send_ix_expect_err(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &escrow::instruction::InitializePolicy {
                policy_id: POLICY_ID,
                holder: holder.pubkey(),
                expiry: 1_700_000_000,
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
}

#[test]
fn test_initialize_policy_rejects_empty_asset_class() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    let (policy, _) = policy_pda(&program_id);
    send_ix_expect_err(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &escrow::instruction::InitializePolicy {
                policy_id: POLICY_ID,
                holder: holder.pubkey(),
                expiry: POLICY_EXPIRY,
                asset_class: [0u8; 32],
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
}
