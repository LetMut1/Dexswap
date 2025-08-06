use {
    crate::{
        error::Error,
        processor::Processor,
    },
    solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
    solana_program_error::ToStr,
};
solana_program::entrypoint!(process_instruction);
fn process_instruction(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    if let Err(program_error) = Processor::process(program_id, accounts, instruction_data) {
        solana_program::msg!("{}",program_error.to_str::<Error>());
        return Err(program_error);
    }
    Ok(())
}
