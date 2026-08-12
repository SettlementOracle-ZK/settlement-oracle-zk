use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{clock::Clock, instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const TEST_NOW: i64 = 1_700_000_000;

fn asset_class_bytes(s: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = s.as_bytes();
    out[..bytes.len()].copy_from_slice(bytes);
    out
}

fn setup_svm() -> (LiteSVM, Keypair, Pubkey) {
    let program_id = escrow::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/escrow.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = TEST_NOW;
    svm.set_sysvar::<Clock>(&clock);

    (svm, payer, program_id)
}

fn policy_pda(program_id: &Pubkey, policy_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[escrow::constants::POLICY_SEED, policy_id.as_ref()],
        program_id,
    )
}

fn create_policy_ix(
    program_id: Pubkey,
    authority: Pubkey,
    policy: Pubkey,
    policy_id: Pubkey,
    holder: Pubkey,
    expiry: i64,
    asset_class: [u8; 32],
) -> Instruction {
    Instruction::new_with_bytes(
        program_id,
        &escrow::instruction::CreatePolicy {
            policy_id,
            holder,
            expiry,
            asset_class,
        }
        .data(),
        escrow::accounts::CreatePolicy {
            authority,
            policy,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn send_ix(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instruction: Instruction,
) -> Result<litesvm::types::TransactionMetadata, litesvm::types::FailedTransactionMetadata> {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx)
}

#[test]
fn test_create_policy_happy_path() {
    let (mut svm, payer, program_id) = setup_svm();
    let policy_id = Keypair::new().pubkey();
    let holder = Keypair::new().pubkey();
    let asset_class = asset_class_bytes("agriculture_climate");
    let expiry = TEST_NOW + 86_400;

    let (policy, bump) = policy_pda(&program_id, &policy_id);
    let ix = create_policy_ix(
        program_id,
        payer.pubkey(),
        policy,
        policy_id,
        holder,
        expiry,
        asset_class,
    );

    assert!(send_ix(&mut svm, &payer, ix).is_ok());

    let policy_account = svm.get_account(&policy).unwrap();
    let mut data: &[u8] = &policy_account.data;
    let state = escrow::state::PolicyAccount::try_deserialize(&mut data).unwrap();

    assert_eq!(state.policy_id, policy_id);
    assert_eq!(state.authority, payer.pubkey());
    assert_eq!(state.holder, holder);
    assert_eq!(state.expiry, expiry);
    assert_eq!(state.asset_class, asset_class);
    assert_eq!(state.created_at, TEST_NOW);
    assert_eq!(state.bump, bump);
}

#[test]
fn test_create_policy_invalid_expiry() {
    let (mut svm, payer, program_id) = setup_svm();
    let policy_id = Keypair::new().pubkey();
    let holder = Keypair::new().pubkey();
    let asset_class = asset_class_bytes("agriculture_climate");
    let expiry = TEST_NOW; // not strictly greater than now

    let (policy, _) = policy_pda(&program_id, &policy_id);
    let ix = create_policy_ix(
        program_id,
        payer.pubkey(),
        policy,
        policy_id,
        holder,
        expiry,
        asset_class,
    );

    assert!(send_ix(&mut svm, &payer, ix).is_err());
}

#[test]
fn test_create_policy_invalid_asset_class() {
    let (mut svm, payer, program_id) = setup_svm();
    let policy_id = Keypair::new().pubkey();
    let holder = Keypair::new().pubkey();
    let asset_class = [0u8; 32];
    let expiry = TEST_NOW + 86_400;

    let (policy, _) = policy_pda(&program_id, &policy_id);
    let ix = create_policy_ix(
        program_id,
        payer.pubkey(),
        policy,
        policy_id,
        holder,
        expiry,
        asset_class,
    );

    assert!(send_ix(&mut svm, &payer, ix).is_err());
}
