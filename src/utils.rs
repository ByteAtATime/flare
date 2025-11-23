pub fn open_url(url: &str) -> Result<(), std::io::Error> {
    webbrowser::open(url)
}
