use std::env;
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

const REPLACE_NODES: &str = "@nodes";
const REPLACE_RANKS: &str = "@ranks";
const REPLACE_THREADS: &str = "@threads";

const DELAY_MS: u64 = 1000;

#[derive(Debug)]
struct Combo {
    nodes: usize,
    ranks: usize,
    threads: usize,
}

impl Combo {
    fn new(nodes: usize, ranks: usize, threads: usize) -> Combo {
        Combo {
            nodes,
            ranks,
            threads,
        }
    }
        
    fn gen_combos_single_node() -> Vec<Combo> {
        let mut r = Vec::new();
        for i in &FACTORS {
            r.push(Self::new(1, *i, MULTIPLE / *i))
        }
        r
    }

    fn gen_from_args() -> Vec<Combo> {
        let mut r = Vec::new();
        for (i, arg) in env::args().enumerate() {
            if i != 0 {
                r.push(Self::parse_arg(&arg))
            }
        }
      
        r
    }

    fn parse_arg(arg: &str) -> Combo {
        let pieces: Vec<_> = arg.split(',').collect();
        let mut nodes = 0;
        let mut ranks = 0;
        let mut threads = 0;

        for (i, piece) in pieces.iter().enumerate() {
            //println!("{} {}", i, piece);
            match i {
                0 => nodes = piece.parse().unwrap(),
                1 => ranks = piece.parse().unwrap(),
                2 => threads = piece.parse().unwrap(),
                _ => panic!("Invalid number of pieces in arg"),
            }
        }

        //println!("{:?}", Combo::new(nodes, ranks, threads));
        Combo::new(nodes, ranks, threads)
    }

    fn stringify(&self) -> (String, String, String) {
        (self.nodes.to_string(),
         self.ranks.to_string(),
         self.threads.to_string())
    }
}

#[derive(Debug)]
struct Template {
    content: String,
}

impl Template {
    fn new<P: AsRef<Path>>(filepath: P) -> Template {
        let content = fs::read_to_string(filepath).unwrap();

        Template {
            content,
        }
    }

    fn finish(&self, combo: &Combo) -> String {
        let (nodes_str, ranks_str, threads_str) = combo.stringify();
        //println!("{}, {}, {}", nodes_str, ranks_str, threads_str);
        let mut result = self.content.replace(REPLACE_NODES, &nodes_str);
        result = result.replace(REPLACE_RANKS, &ranks_str);
        result.replace(REPLACE_THREADS, &threads_str)
    }
}

fn save_script(combo: &Combo, content: &String) -> String {
    let filename = format!("{}/run_{}nodes_{}ranks_{}threads.sh",
            PARENT, combo.nodes, combo.ranks, combo.threads);
    
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
            .arg(filename)
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

    let combos = if env::args().len() == 1 {
        // No args: do 1 node, all combos
        Combo::gen_combos_single_node()
    } else {
        // Any args: do all specified combos in format nodes,ranks,threads
        Combo::gen_from_args()
    };

    println!("{:?}\n", combos);

    //for i in &combos {
    //    submit(&save_script(i, &template.finish(i)));
    //    delay();
    //}

    for i in &combos {
        save_script(i, &template.finish(i));
    }
}
