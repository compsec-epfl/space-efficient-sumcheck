<h1 align="center">Blendy 🍹: a space-efficient sumcheck algorithm</h1>

<p align="center">
    <a href="https://github.com/compsec-epfl/space-efficient-sumcheck/blob/main/LICENSE-APACHE"><img src="https://img.shields.io/badge/license-APACHE-blue.svg"></a>
    <a href="https://github.com/compsec-epfl/space-efficient-sumcheck/blob/main/LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

This library was developed as part of the [eprint.iacr.org/2024/XXX](eprint.iacr.org/2024/XXX) paper using the arkworks ecosystem.

**WARNING:** This is an academic proof-of-concept prototype, and in particular has not received careful code review. This implementation is NOT ready for production use.

## Overview
This library implements the sumcheck protocol using two classical prover algorithms and a new prover algorithm, _Blendy_, that runs in linear time and sublinear space.

[Prover](/src/provers/prover.rs) is a trait implemented by each of the three algorithms<br>
[SpaceProver](/src/provers/space_prover.rs) implements the CormodeTY10 algorithm and runs in superlinear time and uses logarithmic space <br>
[TimeProver](/src/provers/time_prover.rs) implements the VuSBW13 algorithm and runs in linear time and uses linear space <br>
[BlendyProver](/src/provers/blendy_prover.rs) implements Blendy and runs in linear time and uses linear space <br>
[Proof](/src/proof.rs) is a runner that takes a Prover and randomness and runs the protocol to generate a transcript<br>
[Lag Poly](/src/lagrange_polynomial.rs) implements the sequential lagrange polynomial routine described in section 4.1<br>
[Hypercube](/src/hypercube.rs) implements `iter` for boolean hypercube members as a wrapper over a `usize` <br>

## Evaluation
As expected, the asymptotic improvement of BlendyProver translates to significantly lower memory consumption than TimeProver across all configurations tested. TimeProver and BlendyProver have similar runtimes and are orders of magnitude faster than SpaceProver.

<p align="center">
    <img src="assets/evaluation_graphs.png#gh-light-mode-only" alt="Line graph showing runtime and memory consumption of provers for inputs ranging from 15 to 30 variables" width="75%" />
    <img src="assets/evaluation_graphs_inverted.png#gh-dark-mode-only" alt="Line graph showing runtime and memory consumption of provers for inputs ranging from 15 to 30 variables" width="75%" />
</p>

## License
This library is released under the MIT and Apache v2 Licenses.

## Reference paper

[A time-space tradeoff for the sumcheck prover](eprint.iacr.org/2024/XXX)<br>
[Alessandro Chiesa](https://ic-people.epfl.ch/~achiesa/), [Elisabetta Fedele](https://elisabettafedele.github.io), [Giacomo Fenzi](https://gfenzi.io), and [Andrew Zitek-Estrada](https://github.com/z-tech)