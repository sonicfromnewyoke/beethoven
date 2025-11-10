use pinocchio::{
    ProgramResult, account_info::AccountInfo, instruction::Signer, program_error::ProgramError,
};

#[derive(Default)]
pub enum Mode {
    #[default]
    ExactIn,
    ExactOut,
}

pub trait Swap<'info> {
    /// Protocol-specific accounts required for the swap CPI
    type Accounts;

    /// Execute a swap with PDA signing capability
    ///
    /// # Arguments
    /// * `ctx` - Protocol-specific account context
    /// * `amount_in` - Amount to swap in
    /// * `amount_out` - Amount to receive
    /// * `mode` - Swap mode (ExactIn or ExactOut)
    /// * `signer_seeds` - Seeds for PDA signing
    fn swap_signed(
        ctx: &Self::Accounts,
        amount_in: u64,
        amount_out: u64,
        mode: Mode,
        signer_seeds: &[Signer],
    ) -> ProgramResult;

    /// Execute a swap without signing (user is direct signer)
    ///
    /// # Arguments
    /// * `ctx` - Protocol-specific account context
    /// * `amount_in` - Amount to swap in
    /// * `amount_out` - Amount to receive
    /// * `mode` - Swap mode (ExactIn or ExactOut)
    fn swap(ctx: &Self::Accounts, amount_in: u64, amount_out: u64, mode: Mode) -> ProgramResult;
}

pub enum SwapContext<'info> {
    #[cfg(feature = "raydium-cpmm")]
    RaydiumCPMM(crate::programs::raydium_cpmm::RaydiumCPMMSwapAccounts<'info>),
}

impl<'info> Swap<'info> for SwapContext<'info> {
    type Accounts = Self;

    fn swap_signed(
        ctx: &Self::Accounts,
        amount_in: u64,
        amount_out: u64,
        mode: Mode,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        match ctx {
            #[cfg(feature = "raydium-cpmm")]
            SwapContext::RaydiumCPMM(raydium_cpmm_ctx) => {
                crate::programs::raydium_cpmm::RaydiumCPMM::swap_signed(
                    raydium_cpmm_ctx,
                    amount_in,
                    amount_out,
                    mode,
                    signer_seeds,
                )
            }
        }
    }

    fn swap(ctx: &Self::Accounts, amount_in: u64, amount_out: u64, mode: Mode) -> ProgramResult {
        Self::swap_signed(ctx, amount_in, amount_out, mode, &[])
    }
}

pub fn try_from_swap_context<'info>(
    accounts: &'info [AccountInfo],
) -> Result<SwapContext<'info>, ProgramError> {
    let detector_account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    #[cfg(feature = "raydium-cpmm")]
    if detector_account
        .key()
        .eq(&crate::programs::raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID)
    {
        let ctx = crate::programs::raydium_cpmm::RaydiumCPMMSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::RaydiumCPMM(ctx));
    }

    Err(ProgramError::InvalidAccountData)
}

pub fn swap_signed<'info>(
    accounts: &'info [AccountInfo],
    amount_in: u64,
    amount_out: u64,
    mode: Mode,
    signer_seeds: &[Signer],
) -> ProgramResult {
    let ctx = try_from_swap_context(accounts)?;
    SwapContext::swap_signed(&ctx, amount_in, amount_out, mode, signer_seeds)
}

pub fn swap<'info>(
    accounts: &'info [AccountInfo],
    amount_in: u64,
    amount_out: u64,
    mode: Mode,
) -> ProgramResult {
    swap_signed(accounts, amount_in, amount_out, mode, &[])
}
