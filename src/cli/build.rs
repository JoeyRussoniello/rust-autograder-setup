use std::path::Path;
use anyhow::{Context, Result};

use serde_json;
use std::fs::{File, create_dir_all};
use std::io::BufReader;

use crate::utils::{
    ensure_exists, 
    slug_id,
    yaml_quote, 
    YAML_INDENT, 
    YAML_PREAMBLE
};

use crate::types::{AutoTest};

pub fn run(root: &Path) -> Result<()>{
    let autograder_config = root.join("tests").join("autograder.json");
    ensure_exists(&autograder_config)?;
    let tests = read_autograder_config(&autograder_config)?;

    if tests.is_empty() {
        anyhow::bail!("Autograder.json config not configured. Add tests using `auto-setup init`");
    }

    let workflows_dir = root.join(".github").join("workflows");
    create_dir_all(&workflows_dir)
        .with_context(|| format!("Failed to create {}", workflows_dir.to_string_lossy()))?;

    //.yml used instead of .YAML for github classroom compatibility
    let workflow_path = workflows_dir.join("classroom.yml");
    
    let mut yaml_compiler = YAMLAutograder::new();
    yaml_compiler.set_preamble(YAML_PREAMBLE.to_string());
    yaml_compiler.set_tests(tests);
    let workflow_content = yaml_compiler.compile();

    write_workflow(&workflow_path, &workflow_content)?;
    println!("Wrote Configured autograder YAML to {}", workflow_path.to_string_lossy());
    return Ok(());
}

fn read_autograder_config(path: &Path) -> Result<Vec<AutoTest>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let tests = serde_json::from_reader(reader)?;
    return Ok(tests);
}

fn write_workflow(path: &Path, content: &str) -> Result<()> {
    let mut f = File::create(path)
        .with_context(|| format!("Failed to create {}", path.to_string_lossy()))?;
    use std::io::Write;
    f.write_all(content.as_bytes())
        .with_context(|| format!("Failed to write {}", path.to_string_lossy()))?;
    Ok(())
}

pub struct YAMLAutograder {
    pub preamble: String,
    pub autograder_content: String,
    tests: Vec<AutoTest>,
    ids: Vec<String>
}
impl YAMLAutograder {
    fn new() -> Self {
        Self { 
            preamble: String::new(), 
            autograder_content: String::new(),
            tests: Vec::new(),
            ids: Vec::new()
        }
    }

    fn set_preamble(&mut self, preamble: String){
        self.preamble = preamble;
    }

    fn set_tests(&mut self, tests: Vec<AutoTest>){
        self.tests = tests;
        self.ids = Vec::with_capacity(self.tests.len());
    }

    
    fn compile_test_step(&mut self, test: &AutoTest){
        let name = test.name.trim();
        let id = slug_id(name);
        let indent_level = 3;
        self.ids.push(id.clone());
        
        self.insert_autograder_string(format!("- name: {}",name),indent_level);
        self.insert_autograder_string(
            format!(
                "id: {}\nuses: classroom-resources/autograding-command-grader@v1\nwith:",
                id
            ),indent_level + 1
        );

        self.insert_autograder_string(
            format!(
                "test-name: {}\nsetup-command: {}\ncommand: cargo test {}\ntimeout: {}\nmax-score: {}\n", 
                yaml_quote(name),
                yaml_quote(""),
                name,
                test.timeout,
                test.points
            ), indent_level + 2
        );
    }

    fn compile_test_steps(&mut self){
        //Clone tests to avoid an immutable borrow on self
        let tests = self.tests.clone();
        for test in tests.iter(){
            self.compile_test_step(test);
            self.autograder_content.push_str("\n");
        }
    }

    fn compile_test_reporter(&mut self){
        let indent_level = 3;
        self.insert_autograder_string("- name: Autograding Reporter".to_string(), indent_level);
        self.insert_autograder_string(
            "uses: classroom-resources/autograding-grading-reporter@v1\nenv:".to_string(),
            indent_level + 1
        );

        let ids = self.ids.clone();
        for id in ids.iter(){
            let env_key = format!("{}_RESULTS", id.to_uppercase());
            self.insert_autograder_string(
                format!("{}: \"${{{{steps.{}.outputs.result}}}}\"", env_key, id), 
                indent_level + 2
            );
        }

        self.insert_autograder_string("with:".to_string(), indent_level + 1);
        self.insert_autograder_string(
            format!("runners: {}", self.ids.join(",")), 
            indent_level + 2
        );
    }

    fn insert_autograder_string(&mut self, s: String, indent_level: usize){
        let indent = YAML_INDENT.repeat(indent_level);
        //? Could raise error on multi-lines to avoid undetermined behavior
        for line in s.lines(){
            self.autograder_content.push_str(&format!("{}{}\n", indent, line));
        }
    }

    fn compile(&mut self) -> String{
        self.autograder_content.clear();
        self.autograder_content.push_str(&self.preamble);
        self.compile_test_steps();
        self.compile_test_reporter();
        return self.autograder_content.to_string();
    }
}