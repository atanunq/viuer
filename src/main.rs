use std::io::prelude::*;

fn main() {
    let s = "/home/atanunq/repos/rust/viu/img/bfa.jpg";

    let kek = image::open(s).unwrap();
    let rgba = kek.to_rgba();
    let x = rgba.as_raw();

    // TODO: make sure this is fine when many files are displayed consecutively
    let (mut tmpfile, path) = tempfile::Builder::new()
        .prefix(".tmp.viuer.")
        .rand_bytes(1)
        .tempfile()
        .unwrap()
        .keep()
        .unwrap();

    tmpfile.write_all(x).unwrap();
    tmpfile.flush().unwrap();

    // let mut buf = Vec::new();
    tmpfile.seek(std::io::SeekFrom::Start(0)).unwrap();
    // tmpfile.read_to_end(&mut buf).unwrap();
    // println!("{:?}", buf);

    let s = format!(
        "\x1b_Gf=32,s=2560,v=1440,c=100,r=30,a=T,t=f;{}\x1b\\",
        base64::encode(path.to_str().unwrap())
    );
    print!("{}", s);
    std::io::stdout().flush().unwrap();

    // Rust deletes the file before kitty had a chance to read it
    // std::thread::sleep(std::time::Duration::from_millis(10));

    // println!("{:?}", buf);

    // print!("\x1b_Gi=31,s=1,v=1,a=q,t=d,f=24{};\x1b\\", path);

    // let mut config = Config {
    //     x: 60,
    //     y: 0,
    //     width: Some(40),
    //     height: Some(20),
    //     ..Default::default()
    // };

    // print_from_file("../viu/img/bfa", &config).unwrap();

    // config = Config {
    //     x: 61,
    //     y: 1,
    //     ..config
    // };

    // print_from_file("../viu/img/bfa", &config).unwrap();

    // config = Config {
    //     x: 62,
    //     y: 0,
    //     width: Some(40),
    //     height: Some(40),
    //     transparent: true,
    //     ..config
    // };

    // print_from_file("../viu/img/snake.png", &config).unwrap();

    // config = Config {
    //     x: 20,
    //     y: 0,
    //     width: Some(40),
    //     height: Some(20),
    //     ..config
    // };

    // print_from_file("../viu/img/bfa", &config).unwrap();
}
