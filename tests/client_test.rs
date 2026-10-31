use bitchord::ytmusic::YtMusicClient;

#[tokio::test]
async fn test_client_search_and_stream() {
    let client = YtMusicClient::new();

    // 1. Ensure visitor data
    let visitor = client.ensure_visitor_data().await;
    println!("Visitor data present: {}", visitor.is_some());
    assert!(visitor.is_some());

    // 2. Search
    let search = client.search("Kendrick Lamar Not Like Us").await.expect("Search should succeed");
    println!("Search returned {} songs", search.songs.len());
    assert!(!search.songs.is_empty());

    let song = &search.songs[0];
    println!("Found song: {} by {}", song.title, song.artist);

    // 3. Resolve direct stream URL
    let (url, stats, ua) = client.get_stream_url(&song.video_id).await.expect("Stream should resolve");
    println!("Resolved stream URL: {}", &url[..80]);
    println!("Codec: {}, Bitrate: {}kbps, Sample Rate: {}Hz", stats.codec, stats.bitrate_kbps, stats.sample_rate_hz);
    println!("User Agent: {}", ua);
    assert!(url.starts_with("https://"));
    assert!(!ua.is_empty());
}
