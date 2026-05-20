Core Concepts

Arco exposes a small set of fundamental abstractions that map directly to the mathematical structures of linear and mixed-integer programming. Understanding these concepts makes the API predictable and helps explain why certain operations are fast while others require more computation.

Variables and bounds

A variable in Arco represents a decision to be made by the solver. It has a name for debugging, bounds that constrain its possible values, and optionally a type that restricts it to integers or binary values.

Bounds are not just constraints. They are intrinsic properties of the variable that affect how the solver treats it. A variable with bounds zero to infinity is treated differently than an unbounded variable with a separate constraint forcing it positive. The bound information propagates into the solver's presolve phase and can tighten the relaxation used in branch-and-bound.

When you create a variable with model.add_variable(), you get back a Variable object. This object is a lightweight handle that references the actual variable storage inside the model. You can use it in expressions, pass it to constraints, or retrieve its value from a solution. It is valid only for the lifetime of the model that created it.

Variable arrays extend this concept to multiple dimensions. Instead of creating variables one by one, you can define an IndexSet that describes the valid indices, then create a VariableArray over that set. This is not just syntactic sugar. The array structure is preserved through normalization and can be exploited by solvers that support structured models.

Expressions and constraints

Expressions in Arco are algebraic combinations of variables and constants. They support the standard arithmetic operators: addition, subtraction, multiplication by constants, and negation. Multiplication of two variables is not supported because Arco is a linear programming framework, not a general nonlinear one.

Expressions are built lazily. When you write 3.0 _ x + 2.0 _ y, you are constructing a small expression tree. This tree is not immediately evaluated. It is stored in the model and normalized later, during the solve phase. The lazy construction allows Arco to perform algebraic simplifications and common subexpression elimination that would be difficult if expressions were eagerly converted to matrix form.

Constraints relate an expression to a bound. The simplest form is an equality or inequality constraint: x + y >= 5.0. Behind the scenes, this creates a constraint object that references the expression and the bound. Like variables, constraints are handles to internal model storage.

There are also specialized constraint types. Indicator constraints allow modeling logical implications: if a binary variable is 1, then a linear constraint must hold. These are handled specially by MIP solvers and are more efficient than big-M formulations for many problems.

The model lifecycle

A model in Arco progresses through distinct phases. Initially it is mutable. You can add variables, modify constraints, and build up the problem structure. This phase is optimized for construction speed. Allocations happen as needed, and the internal representation is optimized for ease of modification rather than compactness.

When you call solve(), the model transitions to a frozen state. No further modifications are allowed. The freeze triggers normalization, which flattens the expression graph into the sparse matrix format that solvers require. This normalization is where much of the computational work happens. It resolves all variable references, evaluates constant expressions, detects infeasibilities in the bound definitions, and constructs the CSR matrix structure.

After normalization, the solver backend takes over. It consumes the normalized representation, converts it to its own internal format if necessary, and executes the solve algorithm. The result is a SolveResult object that provides access to variable values, the objective value, and status information about whether the solve succeeded and if so, whether the solution is optimal or just feasible.

The model remains frozen after solving. You can inspect the solution, extract values, and analyze the results, but you cannot add new variables or constraints without creating a new model. This immutability allows Arco to make strong guarantees about memory stability. Once normalized, the matrix structure will not change, so solution views can safely reference internal storage without fear of invalidation.

For problems that require multiple related solves, Arco provides block composition. You can define blocks that represent subproblems, compose them into larger workflows, inspect each stage, and pass structured outputs between stages. Warm-start vectors have an explicit solve-setting contract: empty starts are accepted as no-ops, and non-empty `primal_start` inputs raise a typed solver-setting error instead of being ignored.

Type stability and error handling

Arco uses the type system to catch errors early. Bounds are not just numbers; they are Bounds objects that distinguish between finite bounds and infinite ones. Index sets are validated at construction time. Variable arrays know their dimensionality and will raise an error if you try to access them with the wrong number of indices.

Error handling follows Rust conventions. Operations that can fail return Result types. The Python bindings translate these to exceptions with descriptive messages. The goal is that type errors and invalid operations are caught before the solve phase, when the error message can pinpoint exactly which variable or constraint is problematic, rather than after, when you are staring at a solver error about row 847 of a matrix you never constructed explicitly.

This upfront validation is part of the resource-conscious design. It is cheaper to catch an error during model construction than to discover it after a costly normalization or solve phase has already run. The fail-fast approach aligns with the broader goal of predictable behavior: if something is wrong, you know immediately, not after minutes of computation.
