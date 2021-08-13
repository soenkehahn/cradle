use cradle::prelude::*;

fn main() {
    let StdoutTrimmed(git_version) = run_output!(%"git --version");
    eprintln!("git version: {}", git_version);
    let (StdoutTrimmed(git_user), Status(status)) = run_output!(%"git config --get user.name");
    if status.success() {
        eprintln!("git user: {}", git_user);
    } else {
        eprintln!("git user not configured");
    }
}
