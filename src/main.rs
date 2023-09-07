mod bluesky;
mod mastodon;

use bluesky::bluesky_setup;
use mastodon::mastodon_setup;
use tokio::sync::mpsc;

#[tokio::main(flavor = "current_thread")]
async fn main() {
	let (bluesky_post_tx, bluesky_post_rx) = mpsc::channel(2);
	let (mastodon_post_tx, mastodon_post_rx) = mpsc::channel(2);
	bluesky_setup(bluesky_post_rx, mastodon_post_tx).await;
	mastodon_setup(mastodon_post_rx).await;

	tokio::signal::ctrl_c().await.unwrap();
}
