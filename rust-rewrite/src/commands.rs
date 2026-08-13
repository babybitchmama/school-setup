use clap::{Subcommand, ValueEnum};

#[derive(Subcommand)]
pub enum Commands {
    /// View next class, assignment, and exam dates from your calendar
    Calendar,

    /// Initialize course directories based on the configuration files
    InitCourses,

    /// Launch Rofi interface for assignments, books, courses, and notes
    Rofi {
        #[command(subcommand)]
        command: RofiCommands,

        /// Flag to indicate whether to use the current course (default) or to ask for a course selection
        #[arg(long)]
        select_course: bool,
    },

    /// Manage thesis notes, compile documents, and pull Samsung notes
    Thesis {
        #[command(subcommand)]
        command: ThesisCommands,
    },

    /// Create, list, edit, and manage figures
    Figures {
        #[command(subcommand)]
        command: FigureCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum RofiCommands {
    /// View all assignments in rofi
    Assignments,

    /// View all books in rofi
    Books,

    /// View all courses in rofi
    Courses,

    /// View all notes in rofi
    Notes,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum NoteActions {
    New,
    List,
}

#[derive(Subcommand, Debug)]
pub enum ThesisCommands {
    /// List/create a note type (configured in the config.yaml file)
    Notes {
        /// Specify the note type to manage (e.g., "brain_dump", "research_notes")
        note_type: String,

        /// Specify the action to perform on the note type
        action: NoteActions,
    },

    /// Compile all/some notes into a single document
    Compile {
        /// Compile brain dump notes
        #[arg(num_args = 1..)]
        targets: Option<Vec<String>>,

        /// Flag to compile EVERYTHING defined in config.yaml
        #[arg(long)]
        all: bool,
    },

    /// Pull Samsung notes into corresponding folders
    Pull,

    /// Show advisor summary from advisor-info.yaml
    Advisor,

    /// List stored papers and metadata from papers.bib
    Papers,
}

#[derive(clap::Args, Debug, Clone)]
pub struct SharedCreateArgs {
    /// The name of the figure file (e.g., --name my-graph)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Open the figure using the tablet workflow
    #[arg(short, long, default_value_t = false)]
    pub tablet: bool,

    /// Path to a custom SVG template
    #[arg(long)]
    pub template: Option<String>,

    /// Do not use any template; create a completely blank file
    #[arg(long, default_value_t = false)]
    pub no_template: bool,
}

#[derive(clap::Args, Debug, Clone)]
pub struct SharedCopyArgs {
    /// Optional explicit figure name to copy (bypasses Rofi selection if provided)
    #[arg(short, long)]
    pub name: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum CreateTarget {
    /// Target university courses
    Notes {
        /// The name of the course (defaults to 'current-course' if omitted)
        course_name: Option<String>,

        #[command(flatten)]
        shared: SharedCreateArgs,
    },

    /// Target thesis sections
    Thesis {
        /// The thesis note type (e.g., meetings, personal-notes)
        note_type: String,

        #[command(flatten)]
        shared: SharedCreateArgs,
    },

    /// Target assignment folders
    Assignments {
        /// The name of the course (defaults to 'current-course' if omitted)
        course_name: Option<String>,

        #[command(flatten)]
        shared: SharedCreateArgs,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CopyTarget {
    /// Target university courses
    Notes {
        /// The name of the course (defaults to 'current-course' if omitted)
        course_name: Option<String>,

        #[command(flatten)]
        shared: SharedCopyArgs,
    },

    /// Target thesis sections
    Thesis {
        /// The thesis note type (e.g., meetings, personal-notes)
        note_type: String,

        #[command(flatten)]
        shared: SharedCopyArgs,
    },

    /// Target assignment folders
    Assignments {
        /// The name of the course (defaults to 'current-course' if omitted)
        course_name: Option<String>,

        #[command(flatten)]
        shared: SharedCopyArgs,
    },
}

#[derive(clap::Args, Debug, Clone)]
pub struct SharedEditArgs {
    /// Open the figure using the tablet workflow
    #[arg(short, long, default_value_t = false)]
    pub tablet: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum EditTarget {
    Notes {
        course_name: Option<String>,
        #[command(flatten)]
        shared: SharedEditArgs,
    },
    Thesis {
        note_type: String,
        #[command(flatten)]
        shared: SharedEditArgs,
    },
    Assignments {
        course_name: Option<String>,
        #[command(flatten)]
        shared: SharedEditArgs,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum FigureCommands {
    /// Create a new Inkscape figure
    Create {
        #[command(subcommand)]
        target: CreateTarget,
    },

    /// Watch a directory and auto-export SVGs to LaTeX (.pdf_tex)
    Watch {},

    /// Copy the LaTeX snippet for a selected figure to the clipboard via Rofi
    Copy {
        #[command(subcommand)]
        target: CopyTarget,
    },

    /// Compiles and previews a selected figure in a LaTeX PDF document
    Preview {
        #[command(subcommand)]
        target: CopyTarget,
    },

    /// Open Rofi to select and edit an existing figure
    Edit {
        #[command(subcommand)]
        target: EditTarget,
    },

    /// Start the X11 shortcuts daemon for Inkscape
    Shortcuts,

    /// Kill background processes (watch, shortcuts)
    Kill {
        /// Which daemon to kill: 'watch', 'shortcuts'. If omitted, kills both.
        daemon: Option<String>,
    },
}
