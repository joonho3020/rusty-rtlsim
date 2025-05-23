# Rusty RTL Simulation

## Low compilation time

- Directly emit cranelift IR from circuit graph
- Enables much faster compile times vs verilator (verilator compilation is bottlenecked by gcc...)


## High performance

- Directly pipe high level design semantics into RTL simulation to achieve higher performance

- To understand what the above means, lets look at an example

- Case 1

```verilog
always @(posedge clock) begin
    if (condition) begin
        my_reg <= value1;
    end else begin
        my_reg <= value2;
    end
end
```

- Case 2


```verilog
always @(posedge clock) begin
    my_reg <= condition ? value1 : value2;
end
```

- Case 1 vs Case 2. 

Which one do you think results in a faster RTL simulation in Verilator/VCS?
The same? Case 1? Case 2?
The answer is case 1.
Although the two circuits have the exact same behavior, case 1 needs to only evaluate value1 or value2 based on condition.
On the other hand, in case 2, value1 & value2 are always evaluated.
More explanation is in the [RTL simulation tutorial- Event Driven RTL Simulation](https://joonho3020.github.io/articles/rtl-simulation.html).

In Chisel, a lot of the conditions are generated as ternary statements (or case 2).
This is because converting if-else blocks makes it easier to build the Chisel compiler.
However, the ease to build things comes at a tradeoff: low RTL simulation performance.
By avoid the Chisel compiler and trying to directly generate RTL simulations, we can avoid this problem that chisel has.


This also enables other optimizations such as deduplication, vectorization, better circuit partitioning etc (but lets worry about this later).

--- 

## Some useful links

https://docs.rs/cranelift-codegen/latest/cranelift_codegen/
https://docs.rs/cranelift-frontend/latest/cranelift_frontend/
https://docs.rs/cranelift-module/latest/cranelift_module/
https://docs.rs/cranelift-jit/latest/cranelift_jit/
