use fee_transparency_core::{FeeAllocation, FeeCategory, FeeStatus, Institution, Payment, BPS_DENOMINATOR};
use nssa_core::{
    account::{Account, AccountWithMetadata},
    program::{AccountPostState, Claim},
};

/// Lock a payment and allocate amounts across active categories by basis points.
///
/// Scaffold note: does not yet move native/custom tokens; wire custody via token
/// chained calls before testnet launch.
pub fn pay_fees(
    mut institution_account: AccountWithMetadata,
    mut payment_account: AccountWithMetadata,
    payer: AccountWithMetadata,
    mut categories: Vec<AccountWithMetadata>,
    payer_name: String,
    reference_number: String,
    amount: u128,
) -> Vec<AccountPostState> {
    assert!(payer.is_authorized, "Payer must authorize payment");
    assert!(amount > 0, "Payment must be greater than 0");
    assert!(
        payment_account.account == Account::default(),
        "Payment account must be uninitialized"
    );
    assert!(
        payment_account.is_authorized,
        "Payment account must be authorized"
    );

    let mut institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert!(institution.is_verified, "Institution not verified");
    assert!(institution.is_active, "Institution not active");

    let mut allocations = Vec::new();
    for category_account in &mut categories {
        let mut category = FeeCategory::try_from(&category_account.account.data)
            .expect("Category account must hold valid FeeCategory data");
        assert_eq!(
            category.institution_id, institution_account.account_id,
            "Category institution mismatch"
        );
        if !category.is_active {
            continue;
        }
        let share = amount
            .checked_mul(u128::from(category.allocated_percentage_bps))
            .expect("allocation overflow")
            / u128::from(BPS_DENOMINATOR);
        category.total_collected = category
            .total_collected
            .checked_add(share)
            .expect("collected overflow");
        category_account.account.data = (&category).into();
        allocations.push(FeeAllocation {
            category_id: category_account.account_id,
            amount: share,
            released: false,
        });
    }

    let payment = Payment {
        institution_id: institution_account.account_id,
        payer: payer.account_id,
        payer_name,
        reference_number,
        amount,
        status: FeeStatus::Locked,
        allocations,
    };

    institution.total_fees_collected = institution
        .total_fees_collected
        .checked_add(amount)
        .expect("institution collected overflow");
    institution_account.account.data = (&institution).into();
    payment_account.account.data = (&payment).into();

    let mut posts = vec![
        AccountPostState::new(institution_account.account),
        AccountPostState::new_claimed_if_default(payment_account.account, Claim::Authorized),
        AccountPostState::new(payer.account),
    ];
    for category_account in categories {
        posts.push(AccountPostState::new(category_account.account));
    }
    posts
}
