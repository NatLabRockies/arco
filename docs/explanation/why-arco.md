Why Arco

Arco is an optimization library for linear and mixed-integer programming designed specifically for environments where memory is scarce and predictability matters. The name stands for Assembled Resource-Constrained Optimization, which captures two design decisions that shape the entire project.

The assembled part refers to how the API combines ideas from multiple optimization ecosystems. Rather than committing fully to one modeling paradigm, Arco takes the expressiveness of algebraic modeling languages, the type safety of modern systems programming, and the pragmatism of Python's scientific stack, then blends them into something that feels familiar but behaves differently.

The resource-constrained part is where Arco diverges from most optimization tools. Memory behavior is treated as a first-class product requirement, not an implementation detail to optimize later. Every allocation is intentional. Stack and heap behavior are carefully considered. The library is designed to be direct about resource limits rather than silently degrading when memory pressure hits.

Memory as a first-class constraint

Most optimization libraries assume that if a problem fits in RAM, it is fair game. They allocate eagerly, grow vectors dynamically, and rely on the garbage collector or reference counting to clean up eventually. This works fine until it does not. On embedded systems, edge devices, or shared compute clusters, the moment of failure is often unpredictable and catastrophic.

Arco takes a different approach. The library uses fixed-capacity data structures where possible, SmallVec for compact storage of small collections, and packed bitflags to minimize per-element overhead. The constraint matrix is stored in CSR format with 32-bit indices by default, not 64-bit, because most real-world optimization problems do not need the extra address space and the memory savings are substantial at scale.

This is not premature optimization. It is acknowledging that optimization problems are often memory-bound before they are CPU-bound. A solver that cannot fit the constraint matrix in cache or that triggers swap thrashing will spend more time waiting on memory than crunching numbers, regardless of how fast the underlying algorithms are.

Comparison with existing tools

Pyomo dominates the Python optimization landscape for good reason. It is flexible, well-documented, and integrates cleanly with the scientific Python ecosystem. But Pyomo is also heavyweight. Models are built as abstract symbolic expressions that get compiled to concrete representations at solve time. This indirection enables powerful features like automatic differentiation and model transformations, but it comes with memory and performance costs that are difficult to predict.

JuMP in Julia takes a different approach, using multiple dispatch and just-in-time compilation to generate efficient solver interfaces. It is fast and expressive, but it requires committing to the Julia ecosystem, which is a non-starter for teams already invested in Python infrastructure.

Arco sits in the middle space. It keeps the Python frontend that researchers and practitioners expect, but backs it with a Rust core that handles the heavy lifting without garbage collection pauses or dynamic dispatch overhead. The API is lower-level than Pyomo's, closer to direct matrix construction, but it provides enough conveniences like NumPy integration and block composition that you are not writing raw CSR triplets by hand.

The tradeoff is explicit. Arco does not try to be the most flexible modeling language, nor the fastest possible implementation. It aims to be predictable. Memory usage is knowable upfront. Performance does not degrade mysteriously as models grow. When you hit a resource limit, you know exactly which constraint or variable array pushed you over.

Who should use Arco

Arco is built primarily for internal use within our organization, and we are open about that. The API is not stable. Edge cases exist. You are welcome to try it, but we make no guarantees about backward compatibility between versions.

That said, if you are working on constrained hardware, need deterministic memory behavior, or find yourself fighting the garbage collector during large-scale optimization, Arco might be worth a look. The HiGHS solver is embedded, so there is nothing extra to install. The Python API is straightforward. And when you do hit limits, the library is designed to fail visibly and immediately rather than degrade silently.

For battle-tested alternatives, consider Pyomo if you need maximum modeling flexibility, or JuMP if you are comfortable in Julia and want compiled performance. Arco is the option you reach for when memory discipline and predictable behavior matter more than feature count.
