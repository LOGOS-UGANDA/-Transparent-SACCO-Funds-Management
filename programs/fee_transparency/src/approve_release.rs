use fee_transparency_core::{Institution, ReleaseRequest, ReleaseStatus};
use nssa_core::{
    account::AccountWithMetadata,
    program::AccountPostState,
};

pub fn approve_release(
    institution_account: AccountWithMetadata,
    mut release_account: AccountWithMetadata,
    approver: AccountWithMetadata,
) -> Vec<AccountPostState> {
    assert!(approver.is_authorized, "Approver must authorize");

    let institution = Institution::try_from(&institution_account.account.data)
        .expect("Institution account must hold valid Institution data");
    assert!(
        institution.approvers.contains(&approver.account_id),
        "Not an institution approver"
    );

    let mut release = ReleaseRequest::try_from(&release_account.account.data)
        .expect("Release account must hold valid ReleaseRequest data");
    assert_eq!(
        release.institution_id, institution_account.account_id,
        "Release institution mismatch"
    );
    assert_eq!(
        release.status,
        ReleaseStatus::Pending,
        "Release is not pending"
    );
    assert!(
        !release.approvers.contains(&approver.account_id),
        "Already approved"
    );
    assert!(
        !release.rejectors.contains(&approver.account_id),
        "Already rejected"
    );

    release.approvers.push(approver.account_id);
    if release.approvers.len() >= usize::from(institution.required_approvals) {
        release.status = ReleaseStatus::Approved;
    }
    release_account.account.data = (&release).into();

    vec![
        AccountPostState::new(institution_account.account),
        AccountPostState::new(release_account.account),
        AccountPostState::new(approver.account),
    ]
}
