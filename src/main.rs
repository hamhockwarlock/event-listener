use sui_sdk::{rpc_types::CheckpointId, SuiClientBuilder};
use sui_sdk::rpc_types::SuiTransactionBlockResponseOptions;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let sui = SuiClientBuilder::default()
        .ws_url("wss://rpc.devnet.sui.io:443")
        .build("https://fullnode.devnet.sui.io:443")
        .await?;

    // A better approach is to read and track this in a cache in the event of an outage from either
    // Sui or this service.
    let mut current_latest_checkpoint_sequence_number = 0;

    // Assumption 1: Checkpoints don't have a way of being subscribed to in the same way
    // transactions do.
    // I also tried `sui.event_api().subscribe_event(EventFilter::All(vec![]))` but did not see a
    // way to filter checkpoints. The API documentation (https://docs.sui.io/references/event-query-and-subscription#checkpoint-event)
    // mention there is a checkpoint event but the
    // SDK seems to not have an EventFilter::Checkpoint.
    // Assumption 2: The below can't be batched into a PTB, which would add to an increased chance
    // of being rate limited. For sake of PoC we will work around this limitation.
    loop {
        let latest_checkpoint_sequence_number = sui
            .read_api()
            .get_latest_checkpoint_sequence_number()
            .await?;

        if latest_checkpoint_sequence_number > current_latest_checkpoint_sequence_number {
            let latest_checkpoint = sui
                .read_api()
                .get_checkpoint(CheckpointId::SequenceNumber(latest_checkpoint_sequence_number))
                .await?;

            if latest_checkpoint.transactions.len() < 1 { continue; }

            let transactions = sui.read_api().multi_get_transactions_with_options(
                    latest_checkpoint.transactions,
                    SuiTransactionBlockResponseOptions{
                        show_balance_changes: true,
                        show_effects: false,
                        show_events: false,
                        show_input: false,
                        show_object_changes: false,
                        show_raw_effects: false,
                        show_raw_input: false
                    }
                ).await?;

            // Here is where we could send transactions to a queue for additional processing.
            // Something like RabbitMQ that would allow some persistence and replicated queues.
            for tx in transactions.iter() {
                if tx.errors.len() > 0 {
                    eprintln!("{:?}", tx.errors);
                }
                println!("{:?}", tx.balance_changes);
            }

            current_latest_checkpoint_sequence_number = latest_checkpoint_sequence_number;
        }
    }
}
