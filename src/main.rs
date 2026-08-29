use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::fs;
use std::io::{self, stdout};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process;
use std::time::SystemTime;
use walkdir::WalkDir;

/// Move a file or directory and leave a symbolic link at the original location.
///
/// Cross-filesystem directory moves are supported via recursive copy.
/// Symlinks can be relative (../../...) or absolute.
#[derive(Parser, Debug)]
#[command(
    name = "lmv",
    version,
    about = "Move + leave symlink (cross-FS dirs, relative paths)",
    long_about = None
)]
struct Args {
    /// Source path (file or directory)
    #[arg(required = false)]
    source: Option<PathBuf>,

    /// Destination path
    #[arg(required = false)]
    destination: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dry run - show what would be done, do not change anything
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Force - overwrite existing destination
    #[arg(short, long)]
    force: bool,

    /// Archive mode: preserve permissions and modification times
    #[arg(short, long)]
    archive: bool,

    /// Show simple progress (file count)
    #[arg(short = 'P', long)]
    progress: bool,

    /// Create a relative symlink (../../...) instead of absolute
    #[arg(short = 'r', long)]
    relative: bool,

    /// Generate shell completions and exit
    #[arg(long = "gen-completions", value_name = "SHELL", value_enum)]
    gen_completions: Option<Shell>,
}

fn main() {
    let args = Args::parse();

    // Handle --gen-completions
    if let Some(shell) = args.gen_completions {
        let mut cmd = Args::command();
        generate(shell, &mut cmd, "lmv", &mut stdout());
        return;
    }

    let src = match args.source.as_ref() {
        Some(s) => s,
        None => {
            Args::command().print_help().ok();
            eprintln!();
            process::exit(1);
        }
    };
    let dst = match args.destination.as_ref() {
        Some(d) => d,
        None => {
            eprintln!("lmv: missing destination path");
            process::exit(1);
        }
    };

    if let Err(e) = run(src, dst, &args) {
        eprintln!("lmv: {}", e);
        process::exit(1);
    }
}

fn run(src: &Path, dst: &Path, args: &Args) -> Result<(), String> {
    if !src.exists() {
        return Err(format!(
            "cannot access '{}': No such file or directory",
            src.display()
        ));
    }

    let src_abs = src
        .canonicalize()
        .map_err(|e| format!("cannot access '{}': {}", src.display(), e))?;

    // Resolve final destination path (mv-like: if dst is existing dir → put inside)
    let dst_final = if dst.exists() && dst.is_dir() {
        let name = src
            .file_name()
            .ok_or_else(|| format!("invalid source path '{}'", src.display()))?;
        dst.join(name)
    } else {
        dst.to_path_buf()
    };

    // Parent of destination must exist
    if let Some(parent) = dst_final.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(format!(
                "cannot move to '{}': No such file or directory",
                dst_final.display()
            ));
        }
    }

    // Collision check
    if dst_final.exists() {
        let same = dst_final
            .canonicalize()
            .map(|p| p == src_abs)
            .unwrap_or(false);
        if !same {
            if args.force {
                if args.verbose {
                    eprintln!("removing existing destination '{}'", dst_final.display());
                }
                if !args.dry_run {
                    remove_path(&dst_final)?;
                }
            } else {
                return Err(format!(
                    "destination '{}' already exists (use -f to overwrite)",
                    dst_final.display()
                ));
            }
        }
    }

    if args.dry_run {
        let link_style = if args.relative { "relative" } else { "absolute" };
        println!(
            "would move '{}' -> '{}' and leave {} symlink",
            src.display(),
            dst_final.display(),
            link_style
        );
        return Ok(());
    }

    let src_was_absolute = src.is_absolute();
    let link_path = src.to_path_buf();

    // Try atomic rename first (same filesystem)
    match fs::rename(src, &dst_final) {
        Ok(()) => {
            if args.verbose {
                println!("{} -> {}", src.display(), dst_final.display());
            }
        }
        Err(e) if is_cross_device(&e) => {
            if args.verbose {
                println!(
                    "cross-filesystem move: {} -> {}",
                    src.display(),
                    dst_final.display()
                );
            }
            copy_path(src, &dst_final, args)?;
            remove_path(src)?;
        }
        Err(e) => {
            return Err(format!("failed to move '{}': {}", src.display(), e));
        }
    }

    let target_abs = dst_final.canonicalize().map_err(|e| {
        format!(
            "moved but cannot resolve new path '{}': {}",
            dst_final.display(),
            e
        )
    })?;

    // Decide symlink target: relative or absolute
    let link_target = if args.relative {
        let link_parent = link_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let link_parent_abs = if src_was_absolute {
            link_parent.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(link_parent))
                .unwrap_or_else(|_| link_parent.to_path_buf())
                .canonicalize()
                .unwrap_or_else(|_| link_parent.to_path_buf())
        };
        path_relative_from(&link_parent_abs, &target_abs)
            .unwrap_or_else(|| target_abs.clone())
    } else {
        target_abs.clone()
    };

    if let Err(e) = symlink(&link_target, &link_path) {
        let _ = fs::rename(&dst_final, &link_path);
        return Err(format!(
            "moved successfully, but failed to create symlink at '{}': {}",
            link_path.display(),
            e
        ));
    }

    if args.verbose {
        println!("symlink: {} -> {}", link_path.display(), link_target.display());
    }

    Ok(())
}

/// Compute a relative path from `from` directory to `to`.
fn path_relative_from(from: &Path, to: &Path) -> Option<PathBuf> {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();

    let mut common = 0;
    for (a, b) in from.iter().zip(to.iter()) {
        if a == b {
            common += 1;
        } else {
            break;
        }
    }

    if common == 0 {
        return None;
    }

    let mut result = PathBuf::new();
    for _ in common..from.len() {
        result.push("..");
    }
    for comp in &to[common..] {
        result.push(comp.as_os_str());
    }
    if result.as_os_str().is_empty() {
        result.push(".");
    }
    Some(result)
}

fn is_cross_device(e: &io::Error) -> bool {
    e.raw_os_error() == Some(18)
}

fn remove_path(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|e| format!("failed to remove '{}': {}", path.display(), e))
    } else {
        fs::remove_file(path)
            .map_err(|e| format!("failed to remove '{}': {}", path.display(), e))
    }
}

fn copy_path(src: &Path, dst: &Path, args: &Args) -> Result<(), String> {
    if src.is_file() || src.is_symlink() {
        copy_one_file(src, dst, args)?;
        return Ok(());
    }

    if !src.is_dir() {
        return Err(format!("unsupported file type: {}", src.display()));
    }

    fs::create_dir_all(dst).map_err(|e| format!("cannot create '{}': {}", dst.display(), e))?;
    if args.archive {
        copy_metadata(src, dst)?;
    }

    let mut count = 0u64;
    let walker = WalkDir::new(src).follow_links(false).into_iter();

    for entry in walker {
        let entry = entry.map_err(|e| format!("walk error: {}", e))?;
        let path = entry.path();

        let rel = path
            .strip_prefix(src)
            .map_err(|e| format!("strip_prefix: {}", e))?;
        if rel.as_os_str().is_empty() {
            continue;
        }

        let target = dst.join(rel);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)
                .map_err(|e| format!("cannot create '{}': {}", target.display(), e))?;
            if args.archive {
                copy_metadata(path, &target)?;
            }
            if args.verbose {
                println!("dir  {}", target.display());
            }
        } else if entry.file_type().is_symlink() {
            let link_target = fs::read_link(path)
                .map_err(|e| format!("read_link '{}': {}", path.display(), e))?;
            symlink(&link_target, &target).map_err(|e| {
                format!(
                    "cannot create symlink '{}' -> '{}': {}",
                    target.display(),
                    link_target.display(),
                    e
                )
            })?;
            if args.verbose {
                println!("link {} -> {}", target.display(), link_target.display());
            }
        } else if entry.file_type().is_file() {
            copy_one_file(path, &target, args)?;
            count += 1;
            if args.progress && count % 50 == 0 {
                eprint!("\r{} files copied...", count);
            }
        } else if args.verbose {
            eprintln!("skipping special file: {}", path.display());
        }
    }

    if args.progress {
        eprintln!("\r{} files copied.   ", count);
    }

    Ok(())
}

fn copy_one_file(src: &Path, dst: &Path, args: &Args) -> Result<(), String> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create parent '{}': {}", parent.display(), e))?;
    }

    fs::copy(src, dst).map_err(|e| {
        format!(
            "copy '{}' -> '{}': {}",
            src.display(),
            dst.display(),
            e
        )
    })?;

    if args.archive {
        copy_metadata(src, dst)?;
    }

    if args.verbose {
        println!("file {}", dst.display());
    }

    Ok(())
}

fn copy_metadata(src: &Path, dst: &Path) -> Result<(), String> {
    let meta = fs::metadata(src).map_err(|e| format!("metadata '{}': {}", src.display(), e))?;
    let perms = meta.permissions();
    fs::set_permissions(dst, perms)
        .map_err(|e| format!("set_permissions '{}': {}", dst.display(), e))?;
    if let Ok(mtime) = meta.modified() {
        let _ = set_mtime(dst, mtime);
    }
    Ok(())
}

fn set_mtime(path: &Path, mtime: SystemTime) -> io::Result<()> {
    let file = fs::File::open(path)?;
    let times = fs::FileTimes::new().set_modified(mtime);
    file.set_times(times)
}
