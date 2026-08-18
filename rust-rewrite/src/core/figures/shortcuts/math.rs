use crate::open_in_neovim;

use super::config::StylesConfig;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct MathMacroManager {
    figures_tmp_dir: PathBuf,
    template_path: PathBuf,
}

impl MathMacroManager {
    pub fn new() -> Self {
        let figures_tmp_dir = PathBuf::from("/tmp/lesson-manager/figures");
        let _ = fs::create_dir_all(&figures_tmp_dir);

        let template_path =
            shellexpand::tilde("~/.config/lesson-manager/figures/math-template.tex").into_owned();

        MathMacroManager {
            figures_tmp_dir,
            template_path: PathBuf::from(template_path),
        }
    }

    pub fn edit_and_compile(
        &self,
        compile_to_svg: bool,
        terminal: &str,
        editor: &str,
        editor_mode: &String,
        styles: &StylesConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let input_file = self.figures_tmp_dir.join("math_input.tex");
        let master_file = self.figures_tmp_dir.join("master.tex");
        let dvi_path = self.figures_tmp_dir.join("master.dvi");
        let svg_path = self.figures_tmp_dir.join("master.svg");

        fs::write(&input_file, "$$")?;

        let file = std::slice::from_ref(&input_file);
        open_in_neovim(
            self.figures_tmp_dir.as_path(),
            file,
            terminal,
            editor,
            editor_mode,
        );

        let raw_content = fs::read_to_string(&input_file)?;
        let cleaned_lines: Vec<&str> = raw_content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        let trimmed = cleaned_lines.join("\n");

        if trimmed.is_empty() || trimmed == "$$" {
            return Err("No math content entered".into());
        }

        let final_string = if !compile_to_svg {
            format!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
            <svg>
              <text
                 style="font-size:{}px; font-family:'{}'; -inkscape-font-specification:'{}, Normal'; fill:#000000; fill-opacity:1; stroke:none;"
                 xml:space="preserve"><tspan sodipodi:role="line">{trimmed}</tspan></text>
            </svg>"#,
                styles.text_config.font_size, styles.text_config.font, styles.text_config.font
            )
        } else {
            if self.template_path.exists() {
                let template_text = fs::read_to_string(&self.template_path)?;
                let final_content = if template_text.contains("%CONTENT%") {
                    template_text.replace("%CONTENT%", &trimmed)
                } else {
                    template_text.replace(
                        r"\end{document}",
                        &format!("{}\n\\end{{document}}", trimmed),
                    )
                };
                fs::write(&master_file, final_content)?;
            } else {
                let preamble = r#"\documentclass[preview]{standalone}
\usepackage{amsmath}
\usepackage{amssymb}
\usepackage{amsfonts}
\begin{document}"#;
                let end = r"\end{document}";
                let fallback = format!(
                    r#"{}
{}
{}"#,
                    preamble, trimmed, end
                );
                fs::write(&master_file, fallback)?;
            }

            let latex_status = Command::new("pdflatex")
                .arg("-interaction=nonstopmode")
                .arg("--output-format=dvi")
                .arg(format!(
                    "-output-directory={}",
                    self.figures_tmp_dir.display()
                ))
                .arg(&master_file)
                .status()?;

            if !latex_status.success() {
                return Err("LaTeX compilation failed".into());
            }

            let dvisvgm_status = Command::new("dvisvgm")
                .arg("--no-fonts")
                .arg(&dvi_path)
                .arg("-o")
                .arg(&svg_path)
                .status()?;

            if !dvisvgm_status.success() {
                return Err("dvisvgm conversion failed".into());
            }

            fs::read_to_string(&svg_path)?
        };

        if let Ok(mut xclip_child) = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .arg("-target")
            .arg("image/x-inkscape-svg")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = xclip_child.stdin.take() {
                let _ = stdin.write_all(final_string.as_bytes());
            }
            let _ = xclip_child.wait();
            println!("Copied math macro to clipboard:\n{}", final_string);
        } else {
            println!("Failed to execute xclip.");
        }

        Ok(final_string)
    }
}
