use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct MathMacroManager {
    temp_dir: PathBuf,
}

impl MathMacroManager {
    pub fn new() -> Self {
        let temp_dir = PathBuf::from("/tmp/lesson-manager/math-macro");
        let _ = fs::create_dir_all(&temp_dir);
        Self { temp_dir }
    }

    pub fn trigger_latex_input(&self) {
        let tex_file = self.temp_dir.join("formula.tex");
        let initial_content = r#"\documentclass[preview]{standalone}
\usepackage{amsmath}
\usepackage{amssymb}
\begin{document}
$\displaystyle 

$
\end{document}
"#;
        let _ = fs::write(&tex_file, initial_content);

        let status = Command::new("alacritty")
            .arg("--class")
            .arg("inkscape-math-popup")
            .arg("-e")
            .arg("nvim")
            .arg(&tex_file)
            .status();

        if status.is_err() || !status.unwrap().success() {
            let _ = Command::new("kitty").arg("nvim").arg(&tex_file).status();
        }

        self.process_and_insert_formula(&tex_file);
    }

    fn process_and_insert_formula(&self, tex_file: &PathBuf) {
        if !tex_file.exists() {
            return;
        }

        let work_dir = &self.temp_dir;
        let _ = Command::new("latex")
            .arg("-interaction=nonstopmode")
            .arg("formula.tex")
            .current_dir(work_dir)
            .status();

        let svg_output = work_dir.join("formula.svg");
        let _ = Command::new("dvisvgm")
            .arg("--no-fonts")
            .arg("--exact")
            .arg("formula.dvi")
            .arg("-o")
            .arg(&svg_output)
            .current_dir(work_dir)
            .status();

        println!("✅ Formula processed.");
    }
}
