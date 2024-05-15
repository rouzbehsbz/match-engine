# Business Logic

- Asset precisions in calculations
- Trade fees calculations
- Add time in force option for orders

# Software

- Event sourcing for orders, trades and balances lifecycle
- Add unit and integrate tests
- Check single thread performance vs multi threaded engine and also single worker vs multi worker async runtime
- If engine runs in single thread. conside using `Rc` over `Arc`
- Find a way to create cleaner test environment
- Consider using `#[inline]` for methods when needed