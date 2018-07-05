use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

const TEMPLATE: &str = "template";
const PARENT: &str = "/home/g1082321/seissol/SeisSol/launch_SeisSol";
const PERMISSIONS: u32 = 0o744;

const MULTIPLE: usize = 24;
const FACTORS: [usize; 8] = [1, 2, 3, 4, 6, 8, 12, 24];

const REPLACE_RANKS: &str = "@ranks";
const REPLACE_THREADS: &str = "@threads";

const DELAY_MS: u64 = 1000;

#[derive(Debug)]
struct Combo {
    ranks: usize,
    threads: usize,
}

impl Combo {
    fn new(ranks: usize, threads: usize) -> Combo {
        Combo {
            ranks,
            threads,
        }
    }
        
    fn gen_combos() -> Vec<Combo> {
        let mut r = Vec::new();
        for i in &FACTORS {
            r.push(Self::new(*i, MULTIPLE / *i))
        }
        r
    }

    fn stringify(&self) -> (String, String) {
        (self.ranks.to_string(), self.threads.to_string())
    }
}

#[derive(Debug)]
struct Template {
    content: String
}

impl Template {
    fn new<P: AsRef<Path>>(filepath: P) -> Template {
        let content = fs::read_to_string(filepath).unwrap();

        Template {
            content,
        }
    }

    fn finish(&self, combo: &Combo) -> String {
        let (ranks_str, threads_str) = combo.stringify();
        //println!("{}, {}", ranks_str, threads_str);
        let result = self.content.replace(REPLACE_RANKS, &ranks_str);
        result.replace(REPLACE_THREADS, &threads_str)
    }
}

fn save_script(combo: &Combo, content: &String) -> String {
    let filename = format!("{}/run_{}ranks_{}threads.sh",
            PARENT, combo.ranks, combo.threads);
    
    //println!("{}", filename);
    
    let mut file = File::create(&filename).unwrap();
    let mut perms = file.metadata().unwrap().permissions();
    perms.set_mode(PERMISSIONS);

    file.set_permissions(perms).unwrap();
    file.write_all(content.as_bytes()).unwrap();

    filename
}

fn submit(filename: &str) {
    let status = Command::new("qsub")
            .args(&[filename, "-q", "scc", "-l",
                        "nodes=1:ppn=24,walltime=24:00:00"])
            .status()
            .expect("Failed to execute");

    assert!(status.success(), "Failed to submit to queue")
}

fn delay() {
    let duration = Duration::from_millis(DELAY_MS);
    thread::sleep(duration);
}

fn main() {
    let template = Template::new(TEMPLATE);

    let combos = Combo::gen_combos();
    //println!("{:?}\n", combos);

    for i in &combos {
        submit(&save_script(i, &template.finish(i)));
        delay();
    }
}
