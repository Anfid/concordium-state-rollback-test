#![no_std]

use concordium_std::*;

#[derive(Reject)]
enum Error {
    NotFound,
}

#[derive(Debug, Serialize, SchemaType, PartialEq, Eq)]
struct DataEntry {
    key: String,
    value: String,
}

#[derive(Debug, Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
struct State<S: HasStateApi> {
    storage: StateMap<String, String, S>,
}

#[init(contract = "StateRollbackTest")]
fn init<S: HasStateApi>(
    _: &impl HasInitContext,
    state_builder: &mut StateBuilder<S>,
) -> InitResult<State<S>> {
    Ok(State {
        storage: state_builder.new_map(),
    })
}

// Insert function is omitted for brevity

/// Update function
///
/// Update value for the given key if entry exists. If entry does not exist, return [Error::NotFound]
#[receive(
    mutable,
    contract = "StateRollbackTest",
    name = "update",
    parameter = "Entry"
)]
fn contract_update<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &mut impl HasHost<State<S>, StateApiType = S>,
) -> ReceiveResult<()> {
    let state = host.state_mut();

    let params = DataEntry::deserial(&mut ctx.parameter_cursor())?;

    match state.storage.insert(params.key, params.value) {
        Some(_) => Ok(()),
        None => Err(Error::NotFound.into()),
    }
}

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_std::test_infrastructure::*;

    #[concordium_test]
    fn test_transactionality() {
        let ctx = TestInitContext::empty();
        let mut state_builder = TestStateBuilder::new();

        // Init the state with contract function
        let state =
            init(&ctx, &mut state_builder).expect_report("Failed during init_StateRollbackTest");

        let mut host = TestHost::new(state, state_builder);

        // Confirm that state is empty
        claim!(host.state().storage.is_empty());

        let params = to_bytes(&DataEntry {
            key: String::from("key"),
            value: String::from("value"),
        });
        let mut ctx = TestReceiveContext::default();
        ctx.set_parameter(&params);

        // Confirm that state is empty before the call
        claim!(host.state().storage.is_empty());

        // Call update function. Error is expected because entry with given key does not exist
        let result = contract_update(&ctx, &mut host);
        claim_eq!(result, Err(Error::NotFound.into()));

        // Confirm that state is unchanged after the unsuccessfull call
        claim!(host.state().storage.is_empty()); // FALSELY SUCCEEDS!

        // Check that no entry was added after the unsuccessfull call
        claim_eq!(
            host.state()
                .storage
                .get(&String::from("key"))
                .map(|x| x.clone()), // No PartialEq, Eq for StateRef?
            None
        ); // FAILS
    }
}
