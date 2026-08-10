# fee_transparency (LEZ program)

Scaffold for the Transparent SACCO fee-lock program on the Logos Execution Zone.

## Intent

Port the product logic from `prototype/contracts/FeeTransparency.sol` to a native LEZ program:

- Register institutions
- Create fee categories with percentage allocations
- Pay and lock fees into categories
- Request and multi-sig approve releases
- Public verification of payment state

Prefer public LEZ account state for payer-visible audit trails.

## Status

Not implemented yet. Next steps:

1. Follow LEZ program tooling (`spel`, `wallet`, reference programs in [lez-programs](https://github.com/logos-blockchain/lez-programs)).
2. Define instructions and account layout for the capabilities above.
3. Deploy to testnet sequencer and exercise via wallet CLI before wiring `modules/sacco_core`.
