// This file shows what a text-matching linter cannot do.

const BASE: u64 = 10;

fn limits(vault: &mut Vault, amount: u64) {
    // Safe, because both sides are small constants. A linter still flags it (false positive).
    let limit = BASE + 5;
    // Really unsafe, but written without spaces around "+", so the linter misses it (false negative).
    vault.total = vault.total+amount;
}
