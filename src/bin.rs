mod options;
use grompt::{format_status, options::get_options};

fn main() {
    let options = get_options();
    let print_error = options.print_error;
    match format_status(options) {
        Err(e) => {
            if print_error {
                eprintln!("{e}");
            }
            std::process::exit(1);
        }
        Ok(res) => println!("{res}"),
    }
}
