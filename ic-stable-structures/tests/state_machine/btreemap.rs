use super::with_state_machine_context;

#[test]
fn should_init_tx_btreemap() {
    with_state_machine_context(|_, ctx| {
        assert!(ctx.get_tx_from_btreemap(0)?.is_some());

        Ok(())
    })
    .unwrap();
}

#[test]
fn should_push_tx_to_btreemap() {
    with_state_machine_context(|_, ctx| {
        ctx.insert_tx_to_btreemap(1, 1, 10)?;

        assert!(ctx.get_tx_from_btreemap(1).unwrap().is_some());

        Ok(())
    })
    .unwrap();
}

#[test]
fn should_persist_btreemap_tx_after_upgrade() {
    with_state_machine_context(|_, ctx| {
        ctx.insert_tx_to_btreemap(1, 1, 10)?;

        assert!(ctx.get_tx_from_btreemap(1)?.is_some());

        super::upgrade_dummy_canister(ctx)?;

        assert!(ctx.get_tx_from_btreemap(1)?.is_some());

        Ok(())
    })
    .unwrap();
}
