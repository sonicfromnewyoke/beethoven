use core::mem::MaybeUninit;

use pinocchio::{
    ProgramResult,
    account_info::AccountInfo,
    cpi::invoke_signed,
    instruction::{AccountMeta, Instruction, Signer},
    program_error::ProgramError,
};

use crate::{Mode, Swap};

pub const RAYDIUM_CPMM_PROGRAM_ID: [u8; 32] = [
    169, 42, 90, 139, 79, 41, 89, 82, 132, 37, 80, 170, 147, 253, 91, 149, 181, 172, 230, 168, 235,
    146, 12, 147, 148, 46, 67, 105, 12, 32, 236, 115,
];
const SWAP_BASE_INPUT_DISCRIMINATOR: [u8; 8] = [2, 218, 138, 235, 79, 201, 25, 102];
const SWAP_BASE_OUTPUT_DISCRIMINATOR: [u8; 8] = [55, 217, 98, 86, 163, 74, 180, 173];

/// Raydium CPMM DEX integration
pub struct RaydiumCPMM;

pub struct RaydiumCPMMSwapAccounts<'info> {
    pub payer: &'info AccountInfo,
    pub authority: &'info AccountInfo,
    pub amm_config: &'info AccountInfo,
    pub pool_state: &'info AccountInfo,
    pub input_token_account: &'info AccountInfo,
    pub output_token_account: &'info AccountInfo,
    pub input_vault: &'info AccountInfo,
    pub output_vault: &'info AccountInfo,
    pub input_token_program: &'info AccountInfo,
    pub output_token_program: &'info AccountInfo,
    pub input_token_mint: &'info AccountInfo,
    pub output_token_mint: &'info AccountInfo,
    pub observation_state: &'info AccountInfo,
}

impl<'info> TryFrom<&'info [AccountInfo]> for RaydiumCPMMSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        let [
            _raydium_cpmm_program,
            payer,
            authority,
            amm_config,
            pool_state,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            input_token_program,
            output_token_program,
            input_token_mint,
            output_token_mint,
            observation_state,
            _remaining_accounts @ ..,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(RaydiumCPMMSwapAccounts {
            payer,
            authority,
            amm_config,
            pool_state,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            input_token_program,
            output_token_program,
            input_token_mint,
            output_token_mint,
            observation_state,
        })
    }
}

impl<'info> Swap<'info> for RaydiumCPMM {
    type Accounts = RaydiumCPMMSwapAccounts<'info>;

    fn swap_signed(
        ctx: &RaydiumCPMMSwapAccounts<'info>,
        amount_in: u64,
        amount_out: u64,
        mode: Mode,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            AccountMeta::readonly_signer(ctx.payer.key()),
            AccountMeta::readonly(ctx.authority.key()),
            AccountMeta::readonly(ctx.amm_config.key()),
            AccountMeta::writable(ctx.pool_state.key()),
            AccountMeta::writable(ctx.input_token_account.key()),
            AccountMeta::writable(ctx.output_token_account.key()),
            AccountMeta::writable(ctx.input_vault.key()),
            AccountMeta::writable(ctx.output_vault.key()),
            AccountMeta::readonly(ctx.input_token_program.key()),
            AccountMeta::readonly(ctx.output_token_program.key()),
            AccountMeta::readonly(ctx.input_token_mint.key()),
            AccountMeta::readonly(ctx.output_token_mint.key()),
            AccountMeta::writable(ctx.observation_state.key()),
        ];

        let account_infos = [
            ctx.payer,
            ctx.authority,
            ctx.amm_config,
            ctx.amm_config,
            ctx.pool_state,
            ctx.input_token_account,
            ctx.output_token_account,
            ctx.input_vault,
            ctx.output_vault,
            ctx.input_token_program,
            ctx.output_token_program,
            ctx.input_token_mint,
            ctx.output_token_mint,
            ctx.observation_state,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 24]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            match mode {
                Mode::ExactIn => {
                    core::ptr::copy_nonoverlapping(SWAP_BASE_INPUT_DISCRIMINATOR.as_ptr(), ptr, 8);
                }
                Mode::ExactOut => {
                    core::ptr::copy_nonoverlapping(SWAP_BASE_OUTPUT_DISCRIMINATOR.as_ptr(), ptr, 8);
                }
            }
            core::ptr::copy_nonoverlapping(amount_in.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(amount_out.to_le_bytes().as_ptr(), ptr.add(16), 8);
        }

        let instruction = Instruction {
            program_id: &RAYDIUM_CPMM_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 24)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)?;

        Ok(())
    }

    fn swap(
        ctx: &RaydiumCPMMSwapAccounts<'info>,
        amount_in: u64,
        amount_out: u64,
        mode: Mode,
    ) -> ProgramResult {
        Self::swap_signed(ctx, amount_in, amount_out, mode, &[])
    }
}
