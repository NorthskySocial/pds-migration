fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/Northsky-Icon_Color.ico");
        res.set("ProductName", "PDS Migration");
        res.set("FileDescription", "PDS Migration");
        res.set("CompanyName", "Northsky Social");
        res.set("LegalCopyright", "MIT License");
        res.compile().unwrap();
    }
}
