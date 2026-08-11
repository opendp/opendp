# Relational Attribution: Essential Implementation Design

## Objective

Extend OpenDP’s relational support so that a query over a multi-table database can preserve protected-owner attribution through relational preprocessing, then convert owner-level adjacency into a bounded row or group metric before applying a DP measurement.

The design should remain consistent with the existing OpenDP programming model:

[
(\text{domain},\text{metric})
\xrightarrow{\text{Transformation}}
(\text{domain},\text{metric})
\xrightarrow{\text{Measurement}}
\text{release}.
]

Relational preprocessing must therefore compile to ordinary OpenDP `Transformation`s. No new public query framework or framework-wide transformation abstraction is required.

---

## Core semantic invariant

Before contribution bounding, every output row occurrence has an exact set of protected owners.

For any owner set (S), erasing (S) from the input must:

1. delete exactly those output occurrences whose support intersects (S);
2. never create a new output occurrence;
3. never change the value of a surviving occurrence.

Equivalently, every accepted preprocessing operation must commute with owner erasure:

[
T(\Erase_S(X))=\Erase_S(T(X)).
]

This is stronger than ordinary 1-stability. It preserves the identity of the erased owner, which is required when independently compiled query branches are recombined by joins, unions, and similar operators.

The stronger invariant should be maintained privately by the relational compiler. Publicly, the compiled result remains an ordinary OpenDP transformation.

---

## Public programming model

A complete relational query is represented as relational syntax and compiled against an attributed database domain:

```rust
pub fn make_stable_relation(
    input_domain: DatabaseDomain,
    input_metric: OwnerDistance,
    query: RelationExpr,
) -> Fallible<
    Transformation<
        DatabaseDomain,
        PrincipalRelationDomain,
        OwnerDistance,
        OwnerDistance,
    >
>;
```

The resulting transformation can be composed normally:

```rust
let query = make_stable_relation(
    database_domain,
    owner_metric,
    relation_expr,
)?;

let bounded = make_contribution_bound(
    query.output_domain(),
    query.output_metric(),
    bound_spec,
)?;

let mechanism = query >> bounded >> measurement;
```

The public pipeline is:

```text
DatabaseDomain / OwnerDistance
    ↓ stable relational query
PrincipalRelationDomain / OwnerDistance
    ↓ contribution bound or other metric transition
DataFrameDomain / row or group metric
    ↓ DP measurement
Release
```

---

## Source database space

The input domain represents the entire database, including all tables that may be needed for visible query execution or attribution.

```rust
pub struct DatabaseDomain {
    schema: DatabaseSchema,
    ownership: CompiledOwnershipPolicy,
}
```

The ownership policy defines:

* the protected identifier type;
* public tables or rows;
* direct owner anchors;
* attribution propagation relationships;
* deletion closure for mandatory references;
* validity constraints that remain satisfied under owner erasure.

The metric is an owner-level add-remove distance:

```rust
pub struct OwnerDistance<Owner> {
    _owner: PhantomData<Owner>,
}
```

One metric step removes one protected owner and every source occurrence attributed to that owner.

Ownership policy belongs to the source metric space. It must not be inferred ad hoc from expressions encountered later in the query.

---

## Principal relation domain

A pre-bounding relational result is not merely a dataframe. It is a bag of visible row occurrences with exact owner support.

Semantically:

[
\mathsf{Rows}
\subseteq
\mathsf{OccurrenceToken}\times\mathsf{Row},
]

[
\mathsf{Support}
\subseteq
\mathsf{OccurrenceToken}\times\mathsf{Owner}.
]

The domain records how this support is represented and which relational facts are available to later constructors:

```rust
pub struct PrincipalRelationDomain {
    row_domain: DataFrameDomain,
    support: SupportDescriptor,
    facts: RelationalFacts,
    occurrence: OccurrenceDescriptor,
}
```

Possession of a `PrincipalRelationDomain` means exact support has already been established. There is no `Unknown` support variant.

### Support representations

The semantic support relation is canonical. Physical representations are interchangeable backend choices:

```rust
pub enum SupportDescriptor {
    Public,
    InlineFactor(Slot),
    InlineFactors(Vec<Slot>),
    InlineOwnerSet(Slot),
    Factorized(FactorizedSupportDescriptor),
    SideRelation(SupportRelationDescriptor),
}
```

The common path should remain cheap:

* public rows use no support storage;
* singleton owners use one factor;
* fixed buyer/seller-style ownership uses several factors.

Owners tables and arbitrary-cardinality ownership may require an inline set, factorized representation, or side attribution plan.

---

## Occurrence identity

Attribution belongs to bag occurrences, not merely row values. Equal duplicate rows must remain distinguishable when they have different support.

Logical occurrence tokens are internal and derived compositionally:

```text
scan            source occurrence token
filter/map      preserve token
flat-map        (source token, local output ordinal)
union-all       (branch tag, branch token)
inner join      (left token, right token)
coalescing      stable equivalence-class token
choice          stable candidate-class or anchor token
```

A physical row index is not generally valid because deletion of an earlier row may renumber later rows.

When support is carried inline, explicit token materialization may be unnecessary. When support is represented by a side relation, the backend must provide a bag-safe correlation strategy. If an unkeyed duplicate source cannot be correlated safely, the constructor must reject.

---

## Internal compiler artifact

The recursive relation parser should maintain the stronger owner-erasure invariant in a private type:

```rust
struct ParsedPrincipalRelation {
    transformation: Transformation<
        DatabaseDomain,
        PrincipalRelationDomain,
        OwnerDistance,
        OwnerDistance,
    >,
    output_domain: PrincipalRelationDomain,
    proof: PrincipalProof,
}
```

Only trusted relational constructors may create this value.

Its invariant is:

```text
The enclosed transformation is owner-erasure equivariant,
not merely 1-stable.
```

At the public boundary, the wrapper is erased and the ordinary transformation is returned.

This avoids introducing a new public `EquivariantTransformation` framework abstraction while retaining the proof needed internally.

---

## Operation-level construction

Relational preprocessing is not one indivisible constructor.

The parser recursively assembles the complete database-rooted transformation from operation-level transformations.

### Source operations

```text
DatabaseDomain
    → PrincipalRelationDomain
```

Examples:

* scan a table;
* instantiate direct support;
* build source attribution through declared relationships.

### Unary relational operations

```text
PrincipalRelationDomain<A>
    → PrincipalRelationDomain<B>
```

Examples:

* filter;
* projection;
* row-local map;
* explode or unnest;
* distinct;
* group-by;
* windows;
* stable choice.

These remain ordinary, independently testable OpenDP transformations.

For example:

```rust
let child = parse_relation(input_domain, input_metric, filter.input)?;

let local_filter = make_principal_filter(
    child.output_domain.clone(),
    filter.predicate,
)?;

ParsedPrincipalRelation {
    transformation: child.transformation >> local_filter.transformation,
    output_domain: local_filter.output_domain,
    proof: PrincipalProof::compose(
        child.proof,
        local_filter.proof,
    ),
}
```

### Binary and branching operations

Joins and unions combine two subqueries evaluated against the same database:

[
T_L:\mathcal D_{\mathrm{db}}\to\mathcal R_L,
\qquad
T_R:\mathcal D_{\mathrm{db}}\to\mathcal R_R.
]

The parser constructs:

[
D\mapsto\mathsf{Combine}(T_L(D),T_R(D)).
]

A binary constructor must accept only privately certified parsed branches, not arbitrary stable transformations:

```rust
fn parse_join(
    left: ParsedPrincipalRelation,
    right: ParsedPrincipalRelation,
    spec: JoinSpec,
) -> Fallible<ParsedPrincipalRelation>;
```

This restriction is necessary because ordinary 1-stability does not prove that both branches preserve the same erased owner.

The resulting join is still an ordinary database-to-relation transformation. No public synchronized-pair domain is required.

---

## Relational proof rules

Constructors should use a deliberately incomplete library of sound local rules.

The general semantic characterization is a specification, not a decision procedure.

### Derivation-preserving operations

Positive bag operations preserve exact support by tracking each derivation:

* scan;
* row-local filter;
* map;
* flat-map;
* `UNION ALL`;
* inner join.

Inner-join support is:

[
\Support(l,r)
=============

\Support(l)\cup\Support(r).
]

### Coalescing operations

Operations that merge alternative witnesses into one output require:

1. a least witness support under inclusion;
2. output-value invariance whenever that least witness survives.

This covers:

* `DISTINCT`;
* `UNION DISTINCT`;
* semi-join;
* grouped aggregation.

The first implementation should accept conservative cases such as equal-support witnesses or owner-homogeneous groups.

### Fallback and absence operations

Operations that emit an output when no witness exists require a persistent witness whenever the output is initially absent.

This covers:

* anti-join;
* unmatched outer-join branches;
* `NOT EXISTS`.

### Choice operations

Operators that expose one selected witness require:

* a deletion-stable priority;
* selected support contained in every competing witness support.

This covers:

* first or last;
* `arg_min` and `arg_max`;
* keep-first deduplication;
* as-of joins;
* nearest-match joins.

### Anchored contextual computation

When one anchor controls output existence but a wider context controls its value, the value must remain invariant under every erasure disjoint from the anchor support.

This covers:

* owner-local windows;
* ranks;
* cumulative aggregates;
* statistics attached to source rows;
* correlated scalar aggregates.

The practical initial rule should accept owner-homogeneous contexts rather than attempt arbitrary semantic invariance proofs.

### Direct metric transitions

Some operations are not principal but have a finite direct stability bound.

These should return ordinary OpenDP transformations under a different output metric.

For example, global `LIMIT k` is generally not principal, but under row symmetric distance it has the conservative map:

[
d\mapsto 2kd.
]

Such an operation ends principal preprocessing unless another proof system is explicitly introduced for its output metric.

---

## Typed relational facts

Proof roles must remain separate:

```rust
pub struct RelationalFacts {
    equalities: EqualityFacts,
    keys: KeyFacts,
    dependencies: DependencyFacts,
    references: ReferenceFacts,
    attribution: AttributionFacts,
    ordering: OrderingFacts,
}
```

Important distinctions:

* equality permits expression substitution;
* a functional dependency does not make two expressions interchangeable;
* a foreign key does not automatically propagate ownership;
* an attribution path does not imply equality;
* uniqueness must respect backend null semantics;
* deterministic ordering is insufficient unless it is deletion stable.

Constructors generate proof obligations and attempt to discharge them using these facts.

```rust
pub enum Obligation {
    SupportContained { subset: SupportExpr, superset: SupportExpr },
    SupportEqual { left: SupportExpr, right: SupportExpr },
    OwnerHomogeneous { partition: Vec<SlotExpr> },
    ContextInvariant { anchor: SupportExpr, context: ContextExpr },
    StableOrdering { partition: Vec<SlotExpr>, order: Vec<OrderExpr> },
    FixedPublicSchema,
    ErasureClosedValidity,
    ExactEquality { left: SlotExpr, right: SlotExpr },
}
```

The proof engine should be intentionally conservative. Failure to prove an obligation results in rejection or use of a separate stable constructor.

Runtime inspection of private data must not be used to decide whether a proof obligation happens to hold.

---

## Contribution-bounding boundary

Contribution bounding consumes a principal relation and changes metric spaces:

```text
PrincipalRelationDomain / OwnerDistance
    → DataFrameDomain / SymmetricDistance
```

or:

```text
PrincipalRelationDomain / OwnerDistance
    → GroupedFrameDomain / GroupDistance
```

Only vetted contribution-bound constructors may assert such a stability map.

Initial supported bounds should focus on:

* singleton-owner row truncation;
* bounded groups per owner;
* bounded rows per owner-group;
* stable public ordering or deletion-stable owner-local ordering.

Arbitrary-cardinality multi-owner support may be propagated before the boundary, but no general useful maximal exact bound exists on unrestricted multi-owner data. Such bounds require structural restrictions, weaker guarantees, another output metric, or rejection.

After the metric transition, support metadata may be discarded.

---

## Error model

Failures should report the unmet proof obligation rather than a generic unsupported-operation error.

Example:

```text
Cannot prove principal group-by.

The grouping key does not determine complete owner support.

Proved:
  account_id -> account_type

Missing:
  account_id -> complete owner support

Possible resolutions:
  - include owner support in the grouping key;
  - declare a trusted attribution dependency;
  - perform owner-local aggregation before contribution bounding;
  - use a separately proved metric-transition constructor.
```

Proof errors must not reveal private attribution data.

---

## What not to build

Do not make the following foundational:

* vectors of owner expressions;
* hidden owner columns;
* generic binding objects mixing proof roles;
* a public `Principal / Stable / Unsupported` enum;
* pre-bound or post-bound stage flags;
* a separate public query-proof framework;
* arbitrary stable transformations accepted as branches of principal joins;
* runtime tests of private frames to establish semantic proof conditions;
* complete whole-plan inference;
* unrestricted multi-owner contribution truncation.

---

## Implementation sequence

### Phase 1: fixed-factor core

Implement:

* attributed `DatabaseDomain`;
* owner-erasure metric;
* `PrincipalRelationDomain`;
* public, singleton, and fixed multi-owner support;
* scan;
* row-local unary operators;
* `UNION ALL`;
* inner join;
* public-dimension left join;
* owner-homogeneous group-by and windows;
* singleton contribution bounds;
* detailed proof errors.

### Phase 2: richer relational proofs

Add:

* distinct and semi-join coalescing rules;
* anti-join and outer-join persistent-witness rules;
* stable-choice operators;
* direct metric transitions such as bounded global limits;
* stronger fact propagation and ordering proofs.

### Phase 3: arbitrary-cardinality attribution

Add:

* owners-table compilation;
* inline owner sets;
* factorized or side-relation support;
* bag-safe occurrence correlation;
* support-plan cost estimation;
* complexity limits and rejection.

### Research-only or deferred

Do not promise:

* complete semantic inference;
* unrestricted whole-plan recovery after non-principal intermediates;
* general `EXCEPT ALL`;
* recursive relational provenance;
* arbitrary UDF certification;
* useful maximal exact contribution bounding for unrestricted multi-owner data.

---

## Essential design statement

The implementation should satisfy the following architectural rule:

> A relational query is recursively compiled from operation-level OpenDP transformations. During compilation, a private proof artifact certifies that each accepted pre-bounding operation commutes with owner erasure. The public result is an ordinary transformation from an attributed database metric space to a principally attributed relation metric space. Contribution bounding then performs an explicit metric transition before the DP measurement.

This preserves OpenDP’s normal domain–metric–transformation architecture without pretending that ordinary numerical stability alone is sufficient for relational branch recombination.
