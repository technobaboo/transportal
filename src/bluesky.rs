use bisky::{
	atproto::{Client, ClientBuilder},
	bluesky::Bluesky,
	lexicon::app::bsky::feed::Post,
	storage::File,
};
use mastodon_async::NewStatus;
use tokio::sync::mpsc;

pub async fn bluesky_setup(
	bluesky_post_rx: mpsc::Receiver<Post>,
	mastodon_post_tx: mpsc::Sender<NewStatus>,
) {
	let client = ClientBuilder::default()
		.session_from_storage(File::new("bluesky_session.json".into()))
		.await
		.build()
		.unwrap();
	tokio::task::spawn(bluesky_to_mastodon(client.clone(), mastodon_post_tx));
	tokio::task::spawn(bluesky_post_loop(client.clone(), bluesky_post_rx));
}

async fn bluesky_to_mastodon(client: Client, post_tx: mpsc::Sender<NewStatus>) {
	let username = dbg!(client.session.as_ref().unwrap().handle.clone());
	let mut bluesky = Bluesky::new(client);
	let mut me_user = bluesky.user(&username).unwrap();
	let mut post_stream = me_user.stream_posts().await.unwrap();
	loop {
		let latest_post = post_stream.next().await.unwrap();
		dbg!(&latest_post);
		if latest_post.value.reply.is_some() {
			continue;
		}
		post_tx
			.send(NewStatus {
				status: Some(latest_post.value.text),
				in_reply_to_id: None,
				media_ids: None,
				sensitive: None,
				spoiler_text: None,
				visibility: None,
				language: None,
				content_type: None,
			})
			.await
			.unwrap();
	}
}

async fn bluesky_post_loop(client: Client, mut post_rx: mpsc::Receiver<Post>) {
	let mut bluesky = Bluesky::new(client.clone());
	let mut me = bluesky.me().unwrap();
	loop {
		let post = post_rx.recv().await.unwrap();
		me.post(post).await.unwrap();
	}
}
