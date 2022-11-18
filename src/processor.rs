use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use borsh::BorshSerialize;
use crate::error::ReviewError;
use crate::instructions::StudentIntroInstruction;
use std::convert::TryInto;
use crate::state::StudentIntroState;


pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = StudentIntroInstruction::unpack(instruction_data)?;

    match instruction {
        StudentIntroInstruction::AddStudentIntro { name, message } => {
            add_student_intro(program_id, accounts, name, message)
        }
        StudentIntroInstruction::UpdateStudentIntro { name, message } => {
            update_student_intro(program_id, accounts, name, message)
        }
    }
}

pub fn update_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    message: String,
) -> ProgramResult {
    msg!("Updating student intro...");

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let pda_account = next_account_info(account_info_iter)?;

    if pda_account.owner != program_id {
        return  Err(ProgramError::IllegalOwner);
    }

    if !initializer.is_signer {
        msg!("Missing required signature");
        return Err(ProgramError::MissingRequiredSignature);
    }

    msg!("Unpacking state student");
    let mut account_data = try_from_slice_unchecked::<StudentIntroState>(&pda_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    let (pda, _bump_seed) = Pubkey::find_program_address(&[
        initializer.key.as_ref(),
        account_data.name.as_bytes().as_ref(),
    ], program_id);

    if pda != *pda_account.key {
        msg!("Invalid seeds for PDA");
        return Err(ReviewError::InvalidPDA.into());
    }

    if !account_data.is_initialized {
        msg!("Account is not initialized");
        return Err(ReviewError::UninitializedAccount.into());
    }

    let total_len: usize = 1 + (4 + account_data.name.len()) + (4 + message.len());
    if total_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(ReviewError::InvalidDataLength.into());
    }

    account_data.name = name;
    account_data.message = message;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn add_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    message: String,
) -> ProgramResult {

    msg!("Adding student intro..");
    msg!("Name: {}", name);
    msg!("Message: {}", message);
    
   // Get Account iterator
   let account_info_iter = &mut accounts.iter();

   // Get accounts
   let initializer = next_account_info(account_info_iter)?;
   let pda_account = next_account_info(account_info_iter)?;
   let system_program = next_account_info(account_info_iter)?;

   if !initializer.is_signer {
    msg!("Missing required signature");
    return Err(ProgramError::MissingRequiredSignature);
    }

   let (pda, bump_seed) = Pubkey::find_program_address(
       &[initializer.key.as_ref(), name.as_bytes().as_ref()],
       program_id,
   );

   if pda != *pda_account.key {
    msg!("Invalid seeds for PDA");
    return Err(ProgramError::InvalidArgument);
   }

   // Calculate account size required
   // let account_len = 1 + (4 + name.len()) + (4 + message.len());
   let account_len: usize = 1000;

   let total_len: usize = 1 + 1 + (4 + name.len()) + (4 + message.len());
   if total_len > 1000 {
       msg!("Data length is larger than 1000 bytes");
       return Err(ReviewError::InvalidDataLength.into());
   }

   // Calculate rent required
   let rent = Rent::get()?;
   let rent_lamports = rent.minimum_balance(account_len);

   // Create the account
   invoke_signed(
       &system_instruction::create_account(
           initializer.key,
           pda_account.key,
           rent_lamports,
           account_len.try_into().unwrap(),
           program_id,
       ),
       &[
           initializer.clone(),
           pda_account.clone(),
           system_program.clone(),
       ],
       &[&[
           initializer.key.as_ref(),
           name.as_bytes().as_ref(),
           &[bump_seed],
       ]],
   )?;

   msg!("PDA created: {}", pda);

   msg!("unpacking state account");
   let mut account_data =
       try_from_slice_unchecked::<StudentIntroState>(&pda_account.data.borrow()).unwrap();
   msg!("borrowed account data");

   account_data.name = name;
   account_data.message = message;
   account_data.is_initialized = true;

   msg!("serializing account");
   account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
   msg!("state account serialized");

   Ok(())
}

