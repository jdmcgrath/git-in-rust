use anyhow::Context;
use clap::{Parser, Subcommand};
use flate2::bufread::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::ffi::CStr;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,
        file_name: String,
    },
}

enum Kind {
    Blob,
}
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    eprintln!("Logs from your program will appear here!");
    // Uncomment this block to pass the first stage
    match args.command {
        Command::Init => init(),
        Command::CatFile {
            pretty_print,
            object_hash,
        } => cat_file(pretty_print, object_hash),
        Command::HashObject { write, file_name } => hash_object(write, file_name),
    }?;
    Ok(())
}

fn init() -> anyhow::Result<()> {
    fs::create_dir(".git").context("create .git directory")?;
    fs::create_dir(".git/objects").context("create .git/objects directory")?;
    fs::create_dir(".git/refs").context("create .git/refs directory")?;
    fs::write(".git/HEAD", "ref: refs/heads/main\n").context("write .git/HEAD")?;
    println!("Initialized git directory");
    Ok(())
}

fn cat_file(pretty_print: bool, object_hash: String) -> anyhow::Result<()> {
    anyhow::ensure!(
        pretty_print,
        "mode must be given without -p, and we don't support pretty print mode"
    );

    let f = std::fs::File::open(format!(
        ".git/objects/{}/{}",
        &object_hash[..2],
        &object_hash[2..]
    ))
    .context("open in .git/objects")?;
    let z = ZlibDecoder::new(f);
    let mut z = BufReader::new(z);
    let mut buf = Vec::new();
    z.read_until(0, &mut buf)
        .context("read header from .git/objects")?;
    let header = CStr::from_bytes_with_nul(&buf)
        .expect("know there is exactly one nul, and it's at the end");
    let header = header
        .to_str()
        .context(".git/objects file header isn't valid UTF-8")?;
    let Some((kind, size)) = header.split_once(' ') else {
        anyhow::bail!(".git/objects file header did not start with a known type: '{header}'");
    };
    let kind = match kind {
        "blob" => Kind::Blob,
        _ => anyhow::bail!("we do not yet know how to print a '{kind}'"),
    };
    let size = size
        .parse::<usize>()
        .context(".git/objects file header has invalid size: {size}")?;
    buf.clear();
    buf.resize(size, 0);
    z.read_exact(&mut buf[..])
        .context("read true contents of .git/objects file")?;
    let n = z
        .read(&mut [0])
        .context("validate EOF in .git/object file")?;
    anyhow::ensure!(n == 0, ".git/object file had {n} trailing bytes");
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    match kind {
        Kind::Blob => stdout
            .write_all(&buf)
            .context("write object contents to stdout")?,
    }
    Ok(())
}

fn hash_object(write: bool, file_name: String) -> anyhow::Result<()> {
    let f = std::fs::File::open(file_name).context("open file")?;
    let mut z = ZlibEncoder::new(BufReader::new(f), Compression::default());
    let mut buf = Vec::new();
    z.read_to_end(&mut buf).context("read file")?;
    let mut hasher = Sha1::new();
    hasher.update(&buf);
    let file_name_hash = hasher.finalize();
    let file_name_hash = hex::encode(file_name_hash);
    eprintln!("{}", file_name_hash);
    if write {
        let file_location = format!(
            ".git/objects/{}/{}",
            &file_name_hash[..2],
            &file_name_hash[2..]
        );
        let mut file = std::fs::File::create(file_location).context("create file")?;
        file.write_all(&buf)
            .context("write file")
            .context("write file to .git/objects")?;
    }

    Ok(())
}
