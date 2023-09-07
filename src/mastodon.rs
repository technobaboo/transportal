use std::io::{stdin, stdout};

use mastodon_async::{
	helpers::{cli, json},
	scopes::Scopes,
	Mastodon, NewStatus, Registration, Result, StatusesRequest,
};
use reqwest::Client;
use tokio::sync::mpsc;

pub async fn mastodon_setup(mastodon_post_rx: mpsc::Receiver<NewStatus>) {
	let mastodon = register().await.unwrap();
	tokio::task::spawn(mastodon_to_bluesky(mastodon.clone()));
	tokio::task::spawn(mastodon_post_loop(mastodon.clone(), mastodon_post_rx));
}

async fn mastodon_to_bluesky(client: Mastodon) {
	let me = client.verify_credentials().await.unwrap();
	// let mut status_request = StatusesRequest::new();
	// status_request.limit(1);
	// let latest_status = client.statuses(&me.id, status_request).await.unwrap();
	// dbg!(&latest_status);
}

async fn mastodon_post_loop(client: Mastodon, mut post_rx: mpsc::Receiver<NewStatus>) {
	loop {
		let post = post_rx.recv().await.unwrap();
		let _ = client.new_status(post).await;
	}
}

async fn register() -> Result<Mastodon> {
	if let Ok(data) = json::from_file("mastodon_session.json") {
		return Ok(Mastodon::new(Client::new(), data));
	}

	let website = read_line("Please enter your mastodon instance url:")?;
	let registration = Registration::new(website.trim())
		.client_name("mastodon-bluesky-crossposter")
		.scopes(Scopes::all())
		.website("https://github.com/dscottboggs/mastodon-async")
		.build()
		.await?;
	let mastodon = cli::authenticate(registration).await?;

	// Save app data for using on the next run.
	json::to_file(&mastodon.data, "mastodon_session.json")?;

	Ok(mastodon)
}

fn read_line(message: impl AsRef<str>) -> Result<String> {
	use std::io::Write;

	print!("{}", message.as_ref());
	stdout().flush()?;

	let mut input = String::new();
	stdin().read_line(&mut input)?;

	Ok(input.trim().to_string())
}
