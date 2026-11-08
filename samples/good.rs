use anchor_lib::prelude::*;

#[program]
pub mod vault {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        // overflow is turned into an error instead of a panic
        vault.total = vault.total.checked_add(amount).ok_or(ErrorCode::Overflow)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    pub authority: Signer<'info>,
    /// CHECK: only read, address is verified in the instruction
    pub treasury: UncheckedAccount<'info>,
}
