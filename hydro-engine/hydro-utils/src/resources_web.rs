pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    let url = format_url(file_name);
    let data = reqwest::get(url)
        .await?
        .bytes()
        .await?
        .to_vec();

    Ok(data)
}

fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let origin = location.origin().unwrap();
    let base = reqwest::Url::parse(&format!("{origin}/", )).unwrap();
    base.join("assets/").unwrap().join(file_name).unwrap()
}