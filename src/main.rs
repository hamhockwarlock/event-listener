use sui_sdk::{rpc_types::CheckpointId, SuiClientBuilder};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // TODO: move this to a seperate file
    let sui = SuiClientBuilder::default()
        .ws_url("wss://rpc.devnet.sui.io:443")
        .build("https://fullnode.devnet.sui.io:443")
        .await?;

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
            .get_checkpoint(CheckpointId::SequenceNumber(
                latest_checkpoint_sequence_number,
            ))
            .await?;

            let a = sui.read_api().multi_get_transaction_blocks(latest_checkpoint.transactions).await?;

            println!("Sequence Number: {}, Transaction Count: {}", latest_checkpoint_sequence_number, a[0]);
            current_latest_checkpoint_sequence_number = latest_checkpoint_sequence_number;
        }
    }
}
