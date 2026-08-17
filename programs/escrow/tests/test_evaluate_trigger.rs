use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{clock::Clock, instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    bytemuck::{bytes_of, Zeroable},
    escrow::{
        pyth_legacy::{PriceInfo, PriceStatus, SolanaPriceAccount, MAGIC, VERSION_2, ACCOUNT_TYPE_PRICE},
        state::{EscrowAccount, EscrowStatus},
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
    solana_account::Account as SolanaAccount,
};

const POLICY_ID: [u8; 32] = [2u8; 32];
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

fn set_clock(svm: &mut LiteSVM, unix_timestamp: i64) {
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = unix_timestamp;
    svm.set_sysvar::<Clock>(&clock);
}

fn install_mock_pyth_feed(
    svm: &mut LiteSVM,
    price: i64,
    conf: u64,
    publish_time: i64,
) -> Pubkey {
    let feed = Keypair::new();
    let mut account = SolanaPriceAccount::zeroed();
    account.magic = MAGIC;
    account.ver = VERSION_2;
    account.atype = ACCOUNT_TYPE_PRICE;
    account.expo = -8;
    account.timestamp = publish_time;
    account.agg = PriceInfo {
        price,
        conf,
        status: PriceStatus::Trading as u8,
        corp_act: 0,
        _pad: [0; 6],
        pub_slot: 1,
    };

    svm.set_account(
        feed.pubkey(),
        SolanaAccount {
            lamports: 1_000_000,
            data: bytes_of(&account).to_vec(),
            owner: Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    feed.pubkey()
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

fn evaluate_trigger_ix(
    program_id: Pubkey,
    authority: Pubkey,
    policy: Pubkey,
    escrow: Pubkey,
    price_feed: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &escrow::instruction::EvaluateTrigger {}.data(),
        escrow::accounts::EvaluateTrigger {
            authority,
            escrow,
            policy,
            price_feed,
        }
        .to_account_metas(None),
    )
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

#[test]
fn test_evaluate_trigger_happy_path_then_payout() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    svm.airdrop(&holder.pubkey(), HOLDER_AIRDROP).unwrap();
    let holder_before = svm.get_account(&holder.pubkey()).unwrap().lamports;

    let now = 1_700_000_000_i64;
    set_clock(&mut svm, now);

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    let price_feed = install_mock_pyth_feed(&mut svm, 50_000_000_000, 1_000_000, now);

    send_ix(
        &mut svm,
        &payer,
        evaluate_trigger_ix(
            program_id,
            payer.pubkey(),
            policy,
            escrow,
            price_feed,
        ),
    );

    assert_eq!(load_escrow(&svm, &escrow).status, EscrowStatus::Triggered);

    send_ix(
        &mut svm,
        &payer,
        payout_ix(program_id, policy, escrow, holder.pubkey()),
    );

    assert_eq!(load_escrow(&svm, &escrow).status, EscrowStatus::Paid);
    let holder_after = svm.get_account(&holder.pubkey()).unwrap().lamports;
    assert_eq!(holder_after, holder_before + DEPOSIT_LAMPORTS);
}

#[test]
fn test_evaluate_trigger_stale_oracle_blocks_trigger_and_payout() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();
    svm.airdrop(&holder.pubkey(), HOLDER_AIRDROP).unwrap();

    let now = 1_700_000_000_i64;
    set_clock(&mut svm, now);

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    let stale_time = now - 120;
    let price_feed = install_mock_pyth_feed(&mut svm, 50_000_000_000, 1_000_000, stale_time);

    send_ix_expect_err(
        &mut svm,
        &payer,
        evaluate_trigger_ix(
            program_id,
            payer.pubkey(),
            policy,
            escrow,
            price_feed,
        ),
    );

    assert_eq!(load_escrow(&svm, &escrow).status, EscrowStatus::Active);

    send_ix_expect_err(
        &mut svm,
        &payer,
        payout_ix(program_id, policy, escrow, holder.pubkey()),
    );
}

#[test]
fn test_evaluate_trigger_low_confidence_fails() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();

    let now = 1_700_000_000_i64;
    set_clock(&mut svm, now);

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    let price_feed = install_mock_pyth_feed(&mut svm, 50_000_000_000, 10_000_000_000, now);

    send_ix_expect_err(
        &mut svm,
        &payer,
        evaluate_trigger_ix(
            program_id,
            payer.pubkey(),
            policy,
            escrow,
            price_feed,
        ),
    );

    assert_eq!(load_escrow(&svm, &escrow).status, EscrowStatus::Active);
}

#[test]
fn test_evaluate_trigger_condition_not_met() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();

    let now = 1_700_000_000_i64;
    set_clock(&mut svm, now);

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    let price_feed = install_mock_pyth_feed(&mut svm, 150_000_000_000, 1_000_000, now);

    send_ix_expect_err(
        &mut svm,
        &payer,
        evaluate_trigger_ix(
            program_id,
            payer.pubkey(),
            policy,
            escrow,
            price_feed,
        ),
    );

    assert_eq!(load_escrow(&svm, &escrow).status, EscrowStatus::Active);
}

#[test]
fn test_evaluate_trigger_respects_pause() {
    let (mut svm, payer, program_id) = setup_svm();
    let holder = Keypair::new();

    let now = 1_700_000_000_i64;
    set_clock(&mut svm, now);

    let (policy, escrow) = fund_policy_and_escrow(&mut svm, &payer, &program_id, &holder.pubkey());
    let price_feed = install_mock_pyth_feed(&mut svm, 50_000_000_000, 1_000_000, now);

    send_ix(
        &mut svm,
        &payer,
        Instruction::new_with_bytes(
            program_id,
            &escrow::instruction::Pause {}.data(),
            escrow::accounts::PauseEscrow {
                authority: payer.pubkey(),
                escrow,
            }
            .to_account_metas(None),
        ),
    );

    send_ix_expect_err(
        &mut svm,
        &payer,
        evaluate_trigger_ix(
            program_id,
            payer.pubkey(),
            policy,
            escrow,
            price_feed,
        ),
    );

    assert_eq!(load_escrow(&svm, &escrow).status, EscrowStatus::Active);
}
