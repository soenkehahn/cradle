use cradle::prelude::*;

fn main() {
    // output git version
    run!(%"git --version");
    // output configured git user
    let (StdoutTrimmed(git_user), Status(status)) = run_output!(%"git config --get user.name");
    if status.success() {
        eprintln!("git user: {}", git_user);
    } else {
        eprintln!("git user not configured");
    }
}
