// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use base::{
    build::Build, coverage::Coverage, disassemble::Disassemble, errmap::Errmap, info::Info,
    new::New, prove::Prove, test::Test,
};
use move_package::BuildConfig;

pub mod base;
pub mod experimental;
pub mod sandbox;

/// Default directory where saved Move resources live
pub const DEFAULT_STORAGE_DIR: &str = "storage";

/// Default directory for build output
pub const DEFAULT_BUILD_DIR: &str = ".";

/// Extension for resource and event files, which are in BCS format
const BCS_EXTENSION: &str = "bcs";

use anyhow::Result;
use clap::Parser;
use move_core_types::{
    account_address::AccountAddress, errmap::ErrorMapping, gas_schedule::CostTable,
    identifier::Identifier,
};
use move_vm_runtime::native_functions::NativeFunction;
use std::path::PathBuf;

type NativeFunctionRecord = (AccountAddress, Identifier, Identifier, NativeFunction);

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Move {
    /// Path to a package which the command should be run with respect to.
    #[clap(long = "path", short = 'p', global = true, parse(from_os_str))]
    package_path: Option<PathBuf>,

    /// Print additional diagnostics if available.
    #[clap(short = 'v', global = true)]
    verbose: bool,

    /// Package build options
    #[clap(flatten)]
    build_config: BuildConfig,
}

/// MoveCLI is the CLI that will be executed by the `move-cli` command
/// The `cmd` argument is added here rather than in `Move` to make it
/// easier for other crates to extend `move-cli`
#[derive(Parser)]
pub struct MoveCLI {
    #[clap(flatten)]
    move_args: Move,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser)]
pub enum Base {}

#[derive(Parser)]
pub enum Command {
    Build(Build),
    Coverage(Coverage),
    Disassemble(Disassemble),
    Errmap(Errmap),
    Info(Info),
    New(New),
    Prove(Prove),
    Test(Test),
    /// Execute a sandbox command.
    #[clap(name = "sandbox")]
    Sandbox {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        #[clap(long, default_value = DEFAULT_STORAGE_DIR, parse(from_os_str))]
        storage_dir: PathBuf,
        #[clap(subcommand)]
        cmd: sandbox::cli::SandboxCommand,
    },
    /// (Experimental) Run static analyses on Move source or bytecode.
    #[clap(name = "experimental")]
    Experimental {
        /// Directory storing Move resources, events, and module bytecodes produced by module publishing
        /// and script execution.
        #[clap(long, default_value = DEFAULT_STORAGE_DIR, parse(from_os_str))]
        storage_dir: PathBuf,
        #[clap(subcommand)]
        cmd: experimental::cli::ExperimentalCommand,
    },
}

pub fn run_cli(
    natives: Vec<NativeFunctionRecord>,
    cost_table: &CostTable,
    error_descriptions: &ErrorMapping,
    move_args: Move,
    cmd: Command,
) -> Result<()> {
    match cmd {
        Command::Build(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Coverage(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Disassemble(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Errmap(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Info(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::New(c) => c.execute_with_defaults(move_args.package_path),
        Command::Prove(c) => c.execute(move_args.package_path, move_args.build_config),
        Command::Test(c) => c.execute(move_args.package_path, move_args.build_config, natives),
        Command::Sandbox { storage_dir, cmd } => cmd.handle_command(
            natives,
            cost_table,
            error_descriptions,
            &move_args,
            &storage_dir,
        ),
        Command::Experimental { storage_dir, cmd } => cmd.handle_command(&move_args, &storage_dir),
    }
}

pub fn move_cli(
    natives: Vec<NativeFunctionRecord>,
    cost_table: &CostTable,
    error_descriptions: &ErrorMapping,
) -> Result<()> {
    let args = MoveCLI::parse();
    run_cli(
        natives,
        cost_table,
        error_descriptions,
        args.move_args,
        args.cmd,
    )
}
