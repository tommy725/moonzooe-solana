use moon::tokio::task;
use solana_sdk::{
    system_instruction,
    transaction::Transaction,
    message::Message,
    signature::Signer,
};
use voting_program::{self, state::VotingState, instruction as voting_instruction};
use crate::solana_helpers;

pub async fn init_voting_state() -> VotingState { 
    let voting_owner_pubkey = solana_helpers::voting_owner_keypair().pubkey();

    if let Some(voting_state) = solana_helpers::request_voting_state().await {
        return voting_state;
    }

    let voting_state_size = VotingState::serialized_size();
    // @TODO_QUESTION is `system_instruction::create_account_with_seed` ok 
    // or is it better to create an account in a program with `Pubkey::find_program_address` + invoke_signed?
    let create_voting_state_account_ix = system_instruction::create_account_with_seed(
        &voting_owner_pubkey, 
        solana_helpers::voting_state_pubkey(), 
        &voting_owner_pubkey, 
        "voting_state", 
        solana_helpers::request_minimum_balance_for_rent_exemption(voting_state_size).await as u64, 
        voting_state_size as u64, 
        &voting_program::id(),
    );

    let init_voting_ix = voting_instruction::init_voting(
        &voting_owner_pubkey, 
    );
    
    let recent_blockhash = solana_helpers::request_recent_blockhash().await;
    println!("recent_blockhash: {}", recent_blockhash);

    let message = Message::new(
        &[create_voting_state_account_ix, init_voting_ix], 
        None
    );
    let transaction = Transaction::new(
        &[solana_helpers::voting_owner_keypair()], 
        message, 
        recent_blockhash
    );

    println!("Waiting for init_voting transaction...");
    task::spawn_blocking(move || {
        solana_helpers::client().send_and_confirm_transaction(&transaction).expect("init_voting transaction failed");
    }).await.expect("init_voting transaction task failed");

    println!("VotingState initialized.");

    solana_helpers::request_voting_state().await.expect("request VotingState failed")
}
