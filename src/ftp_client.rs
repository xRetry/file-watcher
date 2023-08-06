use suppaftp::{
    NativeTlsFtpStream, NativeTlsConnector,
    native_tls::TlsConnector,
};

fn connect() {
    let ftp_stream = NativeTlsFtpStream::connect("eu-central-1.sftpcloud.io:21")
        .expect("Could not connect to FTP server..");
    let mut ftp_stream = ftp_stream
        .into_secure(
            NativeTlsConnector::from(TlsConnector::new()?), 
            "eu-central-1.sftpcloud.io"
        )?;
    ftp_stream.login("ca15da887bb8490fafb83eb5e7a36ca7ex3", "AXRftfDO0CBWAvZLsOLo7l3G1Y9vtUdyesu")?;

    let mut file_reader = std::fs::File::open(path)?;
    ftp_stream.transfer_type(suppaftp::types::FileType::Binary)?;
    ftp_stream.put_file("test.scss", &mut file_reader)?;
    //ftp_stream.mkdir("/test")?;
    let _ = ftp_stream.quit();
}
