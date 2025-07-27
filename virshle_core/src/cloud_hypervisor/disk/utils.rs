use std::path::Path;

// Error Handling
use log::{debug, error, info, trace};
use miette::{IntoDiagnostic, Result};
use virshle_error::{LibError, VirshleError};

/// Convert string to bytes.
pub fn reverse_human_bytes(string: &str) -> Result<u64, VirshleError> {
    if string.strip_suffix("TiB").is_some() {
        let num: &str = string.trim_end_matches("TiB");
        let int: u64 = num.parse()?;
        let int = int * u64::pow(1024, 4);
        Ok(int)
    } else if string.strip_suffix("GiB").is_some() {
        let num: &str = string.trim_end_matches("GiB");
        let int: u64 = num.parse()?;
        let int = int * u64::pow(1024, 3);
        Ok(int)
    } else if string.strip_suffix("MiB").is_some() {
        let num: &str = string.trim_end_matches("GiB");
        let int: u64 = num.parse()?;
        let int = int * u64::pow(1024, 2);
        Ok(int)
    } else if string.strip_suffix("KiB").is_some() {
        let num: &str = string.trim_end_matches("GiB");
        let int: u64 = num.parse()?;
        let int = int * u64::pow(1024, 1);
        Ok(int)
    } else if string.strip_suffix("B").is_some() {
        let num: &str = string.trim_end_matches("GiB");
        let int: u64 = num.parse()?;
        Ok(int)
    } else {
        Err(LibError::builder()
            .msg("Couldn't convert human readable string to bytes")
            .help("Must be of the form 50GiB, 2MiB, 110KiB or 1B")
            .build()
            .into())
    }
}
/// Expand tild "~" in file path.
pub fn shellexpand(relpath: &str) -> Result<String, VirshleError> {
    let source: String = match relpath.starts_with("~") {
        false => relpath.to_owned(),
        true => relpath.replace("~", dirs::home_dir().unwrap().to_str().unwrap()),
    };

    let path = Path::new(&source);
    if path.exists() {
        Ok(source)
    } else {
        let message = format!("Couldn't find file {:#?} expended to {:#?}.", relpath, path);
        error!("{:#?}", message);
        let err = LibError::builder()
            .msg(&message)
            .help("Are you sure the file exist?")
            .build();
        return Err(err.into());
    }
}

pub fn make_empty_file(path: &str) -> Result<(), VirshleError> {
    let mut commands = vec![format!("dd if=/dev/null of={source} bs=1M seek=10")];
    for cmd in commands {
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.state.status {
            Some(Status::Failed) => {
                let message = format!("[disk]: couldn't create empty file.");
                let help = format!(
                    "{} -> {} ",
                    &res.io.stdin.unwrap().trim(),
                    &res.io.stderr.unwrap().trim()
                );
                error!("{}:{}", &message, &help);
            }
            _ => {
                let message = format!("[disk]: created empty file.");
                let help = format!("{}", &res.io.stdin.unwrap().trim(),);
                trace!("{}:{}", &message, &help);
            }
        };
    }
    Ok(())
}
/// Create a sparse file.
/// The fastest method to create file.
/// See: https://unix.stackexchange.com/questions/108858/seek-argument-in-command-dd
pub fn _make_empty_file(path: &str, block_size: &str, file_size: &str) -> Result<(), VirshleError> {
    //dd
    let bs = utils::reverse_human_bytes("1MiB")?;
    let seek = 10;
    let bytes = BytesMut::with_capacity(bs as usize);
    // Truncate file if already exists.
    File::create(&source)?;
    let mut file = OpenOptions::new().write(true).append(true).open(&source)?;
    for i in 0..seek {
        file.write(&bytes);
        file.flush();
    }
    Ok(())
}
/// Create a vfat partition on empty file.
pub fn format_to_vfat(&self) -> Result<&Self, VirshleError> {
    let disk_dir = self.vm.get_disk_dir()?;
    let source = format!("{disk_dir}/pipelight-init");

    let mut commands = vec![format!("mkfs.vfat -F 32 -n INIT {source}")];
    for cmd in commands {
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;
        match res.state.status {
            Some(Status::Failed) => {
                let message = format!("[disk]: couldn't format file to vfat.");
                let help = format!(
                    "{} -> {} ",
                    &res.io.stdin.unwrap().trim(),
                    &res.io.stderr.unwrap().trim()
                );
                error!("{}:{}", &message, &help);
            }
            _ => {
                let message = format!("[disk]: formated init disk to vfat.");
                let help = format!("{}", &res.io.stdin.unwrap().trim(),);
                trace!("{}:{}", &message, &help);
            }
        };
    }
    Ok(self)
}

/// Mount init disk to host filesystem.
pub fn debug_mount(source: &str, target: &str) -> Result<(), VirshleError> {
    // Ensure mounting directory exists and nothing is already mounted.
    umount(target).ok();
    fs::create_dir_all(&target)?;

    let mut commands = vec![];

    // Mount need root priviledge
    #[cfg(debug_assertions)]
    commands.push(format!(
        "sudo mount -t vfat -o loop -o gid=users -o umask=007 {source} {target}"
    ));
    #[cfg(not(debug_assertions))]
    commands.push(format!(
        "mount -t vfat -o loop -o gid=users -o umask=007 {source} {target}"
    ));

    for cmd in commands {
        let mut proc = Process::new();
        let res = proc.stdin(&cmd).run()?;

        match res.state.status {
            Some(Status::Failed) => {
                let message = format!("[disk]: couldn't mount init disk.");
                let help = format!(
                    "{} -> {} ",
                    &res.io.stdin.unwrap().trim(),
                    &res.io.stderr.unwrap().trim()
                );
                error!("{}:{}", &message, &help);
            }
            _ => {
                let message = format!("[disk]: mounted init disk.");
                let help = format!("{}", &res.io.stdin.unwrap().trim(),);
                trace!("{}:{}", &message, &help);
            }
        };
    }
    Ok(())
}

pub fn release_mount(source: &str, target: &str) -> Result<(), VirshleError> {
    // Fetch a listed of supported file systems on this system. This will be used
    // as the fstype to `Mount::new`, as the `Auto` mount parameter.
    let supported = match SupportedFilesystems::new() {
        Ok(supported) => supported,
        Err(why) => {
            eprintln!("failed to get supported file systems: {}", why);
            exit(1);
        }
    };

    // The source block will be mounted to the target directory, and the fstype is likely
    // one of the supported file systems.
    let result = Mount::builder()
        .fstype(FilesystemType::from(&supported))
        .mount(src, dir);

    match result {
        Ok(mount) => {
            let message = format!("[disk]: couldn't mount init disk.");
            let help = format!(
                "{} -> {} ",
                &res.io.stdin.unwrap().trim(),
                &res.io.stderr.unwrap().trim()
            );
            error!("{}:{}", &message, &help);
        }
        Err(why) => {
            let message = format!("[disk]: mounted init disk.");
            let help = format!("{}", &res.io.stdin.unwrap().trim(),);
            trace!("{}:{}", &message, &help);
        }
    };
    Ok(())
}
