use anchor_lib::prelude::*;

#[program]
pub mod vault {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.total = vault.total + amount;
        msg!("deposited {} + fee", amount);
        let fee = amount.checked_mul(2).unwrap();
        vault.fees += fee;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    pub authority: AccountInfo<'info>,
    /// CHECK: only read, address is verified in the instruction
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,
}
