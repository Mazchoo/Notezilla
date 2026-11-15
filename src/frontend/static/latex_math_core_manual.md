# Core LaTeX Math Rendering Manual (`$$ ... $$`)

A comprehensive, production-grade reference manual documenting and demonstrating core LaTeX mathematical notation rendered in display math mode:

```markdown
$$
\text{LaTeX Mathematical Expression}
$$
```

---

## Table of Contents

1. [Display Math Syntax & Parser Rules](#1-display-math-syntax--parser-rules)
2. [Arithmetic, Powers, Roots & Fractions](#2-arithmetic-powers-roots--fractions)
3. [Greek Letters & Math Font Styles](#3-greek-letters--math-font-styles)
4. [Delimiters, Parentheses & Scaled Brackets](#4-delimiters-parentheses--scaled-brackets)
5. [Relations, Logic & Set Theory](#5-relations-logic--set-theory)
6. [Calculus, Limits, Sums & Integrals](#6-calculus-limits-sums--integrals)
7. [Linear Algebra, Vectors & Matrices](#7-linear-algebra-vectors--matrices)
8. [Multi-line Alignments & Piecewise Cases](#8-multi-line-alignments--piecewise-cases)
9. [Accents, Annotations, Over/Under Braces](#9-accents-annotations-overunder-braces)
10. [Spacing, Formatting & Modulo Arithmetic](#10-spacing-formatting--modulo-arithmetic)
11. [Styling, Box Framing & Color Highlights](#11-styling-box-framing--color-highlights)
12. [Master Showcase: Real-World STEM Formulations](#12-master-showcase-real-world-stem-formulations)

---

## 1. Display Math Syntax & Parser Rules

In Markdown and LaTeX renderers (such as KaTeX, MathJax, and standard pandoc/CommonMark processors), **display math** is set off on its own centered line using double dollar signs:

```markdown
$$
\mathcal{L} = \int_{t_1}^{t_2} (T - V) \, dt
$$
```

$$
\mathcal{L} = \int_{t_1}^{t_2} (T - V) \, dt
$$

> [!IMPORTANT]
> **Syntax Best Practices:**
> - Place opening `$$` and closing `$$` on their own dedicated lines to avoid collision with standard markdown parsing rules (e.g. underscores `_` causing unwanted italics).
> - Never leave trailing backslashes at the very end of an equation unless you are continuing inside a multi-line environment (`aligned`, `matrix`).
> - Avoid naked literal dollar signs inside math mode. Use `\text{\$} ` or `\$` when currency is required.

---

## 2. Arithmetic, Powers, Roots & Fractions

### 2.1 Operators & Standard Arithmetic

```latex
$$
a + b - c \pm d \mp e \times f \div g \cdot h \ast i \circ j \bullet k
$$
```

$$
a + b - c \pm d \mp e \times f \div g \cdot h \ast i \circ j \bullet k
$$

### 2.2 Exponents, Indices & Subscripts

```latex
$$
x^2 \quad x_{i,j} \quad x_i^2 \quad {x^a}_b \quad {}_n C_k \quad e^{-\frac{x^2}{2\sigma^2}}
$$
```

$$
x^2 \quad x_{i,j} \quad x_i^2 \quad {x^a}_b \quad {}_n C_k \quad e^{-\frac{x^2}{2\sigma^2}}
$$

### 2.3 Radicals & Roots

```latex
$$
\sqrt{x} \qquad \sqrt[n]{x + y} \qquad \sqrt{1 + \sqrt{1 + \sqrt{1 + x}}}
$$
```

$$
\sqrt{x} \qquad \sqrt[n]{x + y} \qquad \sqrt{1 + \sqrt{1 + \sqrt{1 + x}}}
$$

### 2.4 Fractions & Continued Fractions

| Variant | Command | Purpose |
| :--- | :--- | :--- |
| Standard | `\frac{a}{b}` | Context-aware sizing |
| Display | `\dfrac{a}{b}` | Forces large display-fraction sizing |
| Text | `\tfrac{a}{b}` | Forces compact text-fraction sizing |
| Continued | `\cfrac{a}{b}` | Preserves spacing in nested fractions |

```latex
$$
\frac{a + b}{c + d} \qquad \dfrac{1}{1 + \dfrac{1}{x}} \qquad \tfrac{1}{2} x^2
$$
```

$$
\frac{a + b}{c + d} \qquad \dfrac{1}{1 + \dfrac{1}{x}} \qquad \tfrac{1}{2} x^2
$$

```latex
$$
a_0 + \cfrac{1}{a_1 + \cfrac{1}{a_2 + \cfrac{1}{a_3 + \dots}}}
$$
```

$$
a_0 + \cfrac{1}{a_1 + \cfrac{1}{a_2 + \cfrac{1}{a_3 + \dots}}}
$$

### 2.5 Binomial Coefficients

```latex
$$
\binom{n}{k} = \frac{n!}{k!(n - k)!} \qquad \dbinom{N}{K} \quad \tbinom{n}{k}
$$
```

$$
\binom{n}{k} = \frac{n!}{k!(n - k)!} \qquad \dbinom{N}{K} \quad \tbinom{n}{k}
$$

---

## 3. Greek Letters & Math Font Styles

### 3.1 Lowercase Greek Letters

```latex
$$
\begin{gathered}
\alpha, \; \beta, \; \gamma, \; \delta, \; \epsilon, \; \varepsilon, \; \zeta, \; \eta, \; \theta, \; \vartheta, \; \iota, \; \kappa, \; \lambda, \; \mu, \\
\nu, \; \xi, \; \pi, \; \varpi, \; \rho, \; \varrho, \; \sigma, \; \varsigma, \; \tau, \; \upsilon, \; \phi, \; \varphi, \; \chi, \; \psi, \; \omega
\end{gathered}
$$
```

$$
\begin{gathered}
\alpha, \; \beta, \; \gamma, \; \delta, \; \epsilon, \; \varepsilon, \; \zeta, \; \eta, \; \theta, \; \vartheta, \; \iota, \; \kappa, \; \lambda, \; \mu, \\
\nu, \; \xi, \; \pi, \; \varpi, \; \rho, \; \varrho, \; \sigma, \; \varsigma, \; \tau, \; \upsilon, \; \phi, \; \varphi, \; \chi, \; \psi, \; \omega
\end{gathered}
$$

### 3.2 Uppercase Greek Letters

```latex
$$
\Gamma, \; \Delta, \; \Theta, \; \Lambda, \; \Xi, \; \Pi, \; \Sigma, \; \Upsilon, \; \Phi, \; \Psi, \; \Omega
$$
```

$$
\Gamma, \; \Delta, \; \Theta, \; \Lambda, \; \Xi, \; \Pi, \; \Sigma, \; \Upsilon, \; \Phi, \; \Psi, \; \Omega
$$

### 3.3 Mathematical Typefaces & Alphabets

```latex
$$
\begin{aligned}
\text{Blackboard Bold } (\mathbb{R}): & \quad \mathbb{N} \subset \mathbb{Z} \subset \mathbb{Q} \subset \mathbb{R} \subset \mathbb{C} \subset \mathbb{H} \\
\text{Calligraphic } (\mathcal{L}): & \quad \mathcal{A}, \mathcal{B}, \mathcal{C}, \mathcal{D}, \mathcal{E}, \mathcal{F}, \mathcal{H}, \mathcal{L}, \mathcal{N}, \mathcal{O}, \mathcal{P} \\
\text{Fraktur } (\mathfrak{g}): & \quad \mathfrak{a}, \mathfrak{b}, \mathfrak{g}, \mathfrak{h}, \mathfrak{p}, \mathfrak{sl}, \mathfrak{su}, \mathfrak{A}, \mathfrak{B}, \mathfrak{R} \\
\text{Roman / Upright } (\mathrm{d}): & \quad \mathrm{d}x, \quad \mathrm{e}^{i\pi} + 1 = 0 \\
\text{Bold Face } (\mathbf{v}): & \quad \mathbf{x}, \mathbf{y}, \mathbf{A}, \mathbf{B}, \mathbf{\Sigma} \\
\text{Sans-Serif } (\mathsf{E}): & \quad \mathsf{E}[X], \quad \mathsf{Var}(Y), \quad \mathsf{Cov}(X, Y) \\
\text{Monospace } (\mathtt{id}): & \quad \mathtt{ptr} \to \mathtt{next}
\end{aligned}
$$
```

$$
\begin{aligned}
\text{Blackboard Bold } (\mathbb{R}): & \quad \mathbb{N} \subset \mathbb{Z} \subset \mathbb{Q} \subset \mathbb{R} \subset \mathbb{C} \subset \mathbb{H} \\
\text{Calligraphic } (\mathcal{L}): & \quad \mathcal{A}, \mathcal{B}, \mathcal{C}, \mathcal{D}, \mathcal{E}, \mathcal{F}, \mathcal{H}, \mathcal{L}, \mathcal{N}, \mathcal{O}, \mathcal{P} \\
\text{Fraktur } (\mathfrak{g}): & \quad \mathfrak{a}, \mathfrak{b}, \mathfrak{g}, \mathfrak{h}, \mathfrak{p}, \mathfrak{sl}, \mathfrak{su}, \mathfrak{A}, \mathfrak{B}, \mathfrak{R} \\
\text{Roman / Upright } (\mathrm{d}): & \quad \mathrm{d}x, \quad \mathrm{e}^{i\pi} + 1 = 0 \\
\text{Bold Face } (\mathbf{v}): & \quad \mathbf{x}, \mathbf{y}, \mathbf{A}, \mathbf{B}, \mathbf{\Sigma} \\
\text{Sans-Serif } (\mathsf{E}): & \quad \mathsf{E}[X], \quad \mathsf{Var}(Y), \quad \mathsf{Cov}(X, Y) \\
\text{Monospace } (\mathtt{id}): & \quad \mathtt{ptr} \to \mathtt{next}
\end{aligned}
$$

---

## 4. Delimiters, Parentheses & Scaled Brackets

### 4.1 Automatic Scaling with `\left` and `\right`

```latex
$$
\left( \frac{x^2 + 1}{y^3 - 1} \right) \qquad \left[ \sum_{i=1}^n x_i \right] \qquad \left\{ x \in \mathbb{R} \;\middle|\; x > 0 \right\}
$$
```

$$
\left( \frac{x^2 + 1}{y^3 - 1} \right) \qquad \left[ \sum_{i=1}^n x_i \right] \qquad \left\{ x \in \mathbb{R} \;\middle|\; x > 0 \right\}
$$

### 4.2 Angle Brackets, Norms & Floor/Ceil

```latex
$$
\langle u, v \rangle \qquad |x| \qquad \| \mathbf{v} \|_2 \qquad \lfloor x \rfloor \qquad \lceil x \rceil
$$
```

$$
\langle u, v \rangle \qquad |x| \qquad \| \mathbf{v} \|_2 \qquad \lfloor x \rfloor \qquad \lceil x \rceil
$$

### 4.3 Unmatched Delimiters (Evaluation Bar)

Use `\left.` with a matching `\right|` for derivative evaluations and definite integration limits:

```latex
$$
\left. \frac{\partial f(x, y)}{\partial x} \right|_{x=x_0} \qquad \left. \int x^2 \, dx \right|_{0}^{1} = \left. \frac{x^3}{3} \right|_0^1 = \frac{1}{3}
$$
```

$$
\left. \frac{\partial f(x, y)}{\partial x} \right|_{x=x_0} \qquad \left. \int x^2 \, dx \right|_{0}^{1} = \left. \frac{x^3}{3} \right|_0^1 = \frac{1}{3}
$$

### 4.4 Fixed-Size Manual Delimiters

When auto-scaling with `\left` / `\right` produces disproportionate results, use manual size overrides:

```latex
$$
(x) \quad \big( x \big) \quad \Big( x \Big) \quad \bigg( x \bigg) \quad \Bigg( x \Bigg)
$$
```

$$
(x) \quad \big( x \big) \quad \Big( x \Big) \quad \bigg( x \bigg) \quad \Bigg( x \Bigg)
$$

---

## 5. Relations, Logic & Set Theory

### 5.1 Equality, Equivalence & Ordering

```latex
$$
= \quad \ne \quad < \quad > \quad \le \quad \ge \quad \ll \quad \gg \quad \equiv \quad \approx \quad \sim \quad \simeq \quad \cong \quad \propto \quad \coloneqq
$$
```

$$
= \quad \ne \quad < \quad > \quad \le \quad \ge \quad \ll \quad \gg \quad \equiv \quad \approx \quad \sim \quad \simeq \quad \cong \quad \propto \quad \coloneqq
$$

### 5.2 Set Operations & Logic Symbols

```latex
$$
\forall x, \; \exists y, \; \nexists z \quad \implies \quad P \iff Q \quad \land \quad \lor \quad \neg A \quad \therefore \quad \because
$$
```

$$
\forall x, \; \exists y, \; \nexists z \quad \implies \quad P \iff Q \quad \land \quad \lor \quad \neg A \quad \therefore \quad \because
$$

```latex
$$
x \in A \quad y \notin B \quad A \subset B \quad A \subseteq C \quad A \cap B \quad A \cup B \quad A \setminus B \quad A \times B \quad \emptyset \quad \varnothing
$$
```

$$
x \in A \quad y \notin B \quad A \subset B \quad A \subseteq C \quad A \cap B \quad A \cup B \quad A \setminus B \quad A \times B \quad \emptyset \quad \varnothing
$$

### 5.3 Mappings & Extensible Arrows

```latex
$$
A \to B \qquad f \colon X \hookrightarrow Y \qquad x \mapsto f(x) \qquad A \xleftarrow{\text{pullback}} B \xrightarrow[n \to \infty]{\text{uniform}} C
$$
```

$$
A \to B \qquad f \colon X \hookrightarrow Y \qquad x \mapsto f(x) \qquad A \xleftarrow{\text{pullback}} B \xrightarrow[n \to \infty]{\text{uniform}} C
$$

---

## 6. Calculus, Limits, Sums & Integrals

### 6.1 Derivatives & Differential Operators

```latex
$$
\frac{dy}{dx} \qquad \frac{d^2 y}{dx^2} \qquad \frac{\partial f}{\partial x} \qquad \frac{\partial^2 f}{\partial x \partial y} \qquad \nabla \phi \qquad \nabla \cdot \mathbf{F} \qquad \nabla \times \mathbf{F} \qquad \Delta u
$$
```

$$
\frac{dy}{dx} \qquad \frac{d^2 y}{dx^2} \qquad \frac{\partial f}{\partial x} \qquad \frac{\partial^2 f}{\partial x \partial y} \qquad \nabla \phi \qquad \nabla \cdot \mathbf{F} \qquad \nabla \times \mathbf{F} \qquad \Delta u
$$

### 6.2 Limits & Asymptotics

```latex
$$
\lim_{x \to 0} \frac{\sin x}{x} = 1 \qquad \lim_{x \to 0^+} \ln x = -\infty \qquad \limsup_{n \to \infty} a_n \qquad \liminf_{n \to \infty} a_n
$$
```

$$
\lim_{x \to 0} \frac{\sin x}{x} = 1 \qquad \lim_{x \to 0^+} \ln x = -\infty \qquad \limsup_{n \to \infty} a_n \qquad \liminf_{n \to \infty} a_n
$$

### 6.3 Sums, Products & Coproducts

```latex
$$
\sum_{k=1}^{n} k^2 = \frac{n(n + 1)(2n + 1)}{6} \qquad \prod_{k=1}^{\infty} \left(1 - \frac{1}{p_k^s}\right)^{-1} = \zeta(s) \qquad \coprod_{i \in I} X_i
$$
```

$$
\sum_{k=1}^{n} k^2 = \frac{n(n + 1)(2n + 1)}{6} \qquad \prod_{k=1}^{\infty} \left(1 - \frac{1}{p_k^s}\right)^{-1} = \zeta(s) \qquad \coprod_{i \in I} X_i
$$

### 6.4 Single, Multiple & Contour Integrals

```latex
$$
\int_0^\infty e^{-x^2} \, dx = \frac{\sqrt{\pi}}{2} \qquad \iint_D f(x, y) \, dx \, dy \qquad \iiint_V dV \qquad \oint_C \mathbf{F} \cdot d\mathbf{r} = 0
$$
```

$$
\int_0^\infty e^{-x^2} \, dx = \frac{\sqrt{\pi}}{2} \qquad \iint_D f(x, y) \, dx \, dy \qquad \iiint_V dV \qquad \oint_C \mathbf{F} \cdot d\mathbf{r} = 0
$$

---

## 7. Linear Algebra, Vectors & Matrices

### 7.1 Vectors & Notations

```latex
$$
\vec{v} \qquad \mathbf{v} \qquad \hat{\mathbf{u}} = \frac{\mathbf{u}}{\|\mathbf{u}\|} \qquad \mathbf{a} \cdot \mathbf{b} = \|\mathbf{a}\| \|\mathbf{b}\| \cos\theta \qquad \mathbf{a} \times \mathbf{b}
$$
```

$$
\vec{v} \qquad \mathbf{v} \qquad \hat{\mathbf{u}} = \frac{\mathbf{u}}{\|\mathbf{u}\|} \qquad \mathbf{a} \cdot \mathbf{b} = \|\mathbf{a}\| \|\mathbf{b}\| \cos\theta \qquad \mathbf{a} \times \mathbf{b}
$$

### 7.2 Matrix Environments Catalog

| Environment | Border Style | LaTeX Code |
| :--- | :--- | :--- |
| `matrix` | Plain (no enclosing brackets) | `\begin{matrix} ... \end{matrix}` |
| `pmatrix` | Parentheses `( )` | `\begin{pmatrix} ... \end{pmatrix}` |
| `bmatrix` | Square brackets `[ ]` | `\begin{bmatrix} ... \end{bmatrix}` |
| `Bmatrix` | Curly braces `{ }` | `\begin{Bmatrix} ... \end{Bmatrix}` |
| `vmatrix` | Single vertical lines `| |` (Determinant) | `\begin{vmatrix} ... \end{vmatrix}` |
| `Vmatrix` | Double vertical lines `\| \|` (Matrix Norm) | `\begin{Vmatrix} ... \end{Vmatrix}` |

```latex
$$
\begin{matrix}
a & b \\
c & d
\end{matrix}
\qquad
\begin{pmatrix}
a & b \\
c & d
\end{pmatrix}
\qquad
\begin{bmatrix}
a & b \\
c & d
\end{bmatrix}
\qquad
\begin{Bmatrix}
a & b \\
c & d
\end{Bmatrix}
\qquad
\begin{vmatrix}
a & b \\
c & d
\end{vmatrix}
\qquad
\begin{Vmatrix}
a & b \\
c & d
\end{Vmatrix}
$$
```

$$
\begin{matrix}
a & b \\
c & d
\end{matrix}
\qquad
\begin{pmatrix}
a & b \\
c & d
\end{pmatrix}
\qquad
\begin{bmatrix}
a & b \\
c & d
\end{bmatrix}
\qquad
\begin{Bmatrix}
a & b \\
c & d
\end{Bmatrix}
\qquad
\begin{vmatrix}
a & b \\
c & d
\end{vmatrix}
\qquad
\begin{Vmatrix}
a & b \\
c & d
\end{Vmatrix}
$$

### 7.3 Large Dimensional Matrices with Ellipses

```latex
$$
\mathbf{A} = \begin{bmatrix}
a_{11} & a_{12} & \cdots & a_{1n} \\
a_{21} & a_{22} & \cdots & a_{2n} \\
\vdots & \vdots & \ddots & \vdots \\
a_{m1} & a_{m2} & \cdots & a_{mn}
\end{bmatrix}_{m \times n}
$$
```

$$
\mathbf{A} = \begin{bmatrix}
a_{11} & a_{12} & \cdots & a_{1n} \\
a_{21} & a_{22} & \cdots & a_{2n} \\
\vdots & \vdots & \ddots & \vdots \\
a_{m1} & a_{m2} & \cdots & a_{mn}
\end{bmatrix}_{m \times n}
$$

### 7.4 Augmented Matrix via `array`

```latex
$$
\left[
\begin{array}{ccc|c}
1 & 2 & -1 & 3 \\
0 & 1 & 4 & -2 \\
0 & 0 & 1 & 5
\end{array}
\right]
$$
```

$$
\left[
\begin{array}{ccc|c}
1 & 2 & -1 & 3 \\
0 & 1 & 4 & -2 \\
0 & 0 & 1 & 5
\end{array}
\right]
$$

---

## 8. Multi-line Alignments & Piecewise Cases

### 8.1 Aligned Equations (`aligned`)

The `&` character specifies the alignment anchor, while `\\` creates line breaks:

```latex
$$
\begin{aligned}
(x + y)^3 &= (x + y)(x + y)^2 \\
          &= (x + y)(x^2 + 2xy + y^2) \\
          &= x^3 + 2x^2 y + xy^2 + x^2 y + 2xy^2 + y^3 \\
          &= x^3 + 3x^2 y + 3xy^2 + y^3
\end{aligned}
$$
```

$$
\begin{aligned}
(x + y)^3 &= (x + y)(x + y)^2 \\
          &= (x + y)(x^2 + 2xy + y^2) \\
          &= x^3 + 2x^2 y + xy^2 + x^2 y + 2xy^2 + y^3 \\
          &= x^3 + 3x^2 y + 3xy^2 + y^3
\end{aligned}
$$

### 8.2 Parallel Multi-column Alignment (`alignedat`)

```latex
$$
\begin{alignedat}{2}
x + 3y &= 7,   &\quad 2x - y &= 0 \\
-x + 2y &= 3,  &\quad 4x + 5y &= 14
\end{alignedat}
$$
```

$$
\begin{alignedat}{2}
x + 3y &= 7,   &\quad 2x - y &= 0 \\
-x + 2y &= 3,  &\quad 4x + 5y &= 14
\end{alignedat}
$$

### 8.3 Grouped Unaligned Equations (`gathered`)

```latex
$$
\begin{gathered}
2x + 3y = 12 \\
x^2 + y^2 = 25 \\
x, y \in \mathbb{R}^+
\end{gathered}
$$
```

$$
\begin{gathered}
2x + 3y = 12 \\
x^2 + y^2 = 25 \\
x, y \in \mathbb{R}^+
\end{gathered}
$$

### 8.4 Piecewise Functions (`cases` & `dcases`)

```latex
$$
f(x) = \begin{cases}
\dfrac{\sin x}{x} & \text{if } x \ne 0, \\
1 & \text{if } x = 0.
\end{cases}
$$
```

$$
f(x) = \begin{cases}
\dfrac{\sin x}{x} & \text{if } x \ne 0, \\
1 & \text{if } x = 0.
\end{cases}
$$

---

## 9. Accents, Annotations, Over/Under Braces

### 9.1 Accents on Variables

```latex
$$
\hat{a} \quad \bar{a} \quad \tilde{a} \quad \vec{a} \quad \dot{a} \quad \ddot{a} \quad \acute{a} \quad \grave{a} \quad \check{a} \quad \breve{a} \quad \mathring{a}
$$
```

$$
\hat{a} \quad \bar{a} \quad \tilde{a} \quad \vec{a} \quad \dot{a} \quad \ddot{a} \quad \acute{a} \quad \grave{a} \quad \check{a} \quad \breve{a} \quad \mathring{a}
$$

### 9.2 Wide Accents & Horizontal Over/Under Lines

```latex
$$
\widehat{A + B} \qquad \widetilde{X_1 + X_2} \qquad \overline{z} = x - iy \qquad \underline{u + v}
$$
```

$$
\widehat{A + B} \qquad \widetilde{X_1 + X_2} \qquad \overline{z} = x - iy \qquad \underline{u + v}
$$

### 9.3 Overbrace & Underbrace

```latex
$$
\overbrace{a + b + \dots + z}^{26 \text{ letters}} \qquad \underbrace{x_1 + x_2 + \dots + x_k}_{k \text{ terms}} = S
$$
```

$$
\overbrace{a + b + \dots + z}^{26 \text{ letters}} \qquad \underbrace{x_1 + x_2 + \dots + x_k}_{k \text{ terms}} = S
$$

### 9.4 Stacking Symbols (`\overset` & `\underset`)

```latex
$$
A \overset{\text{def}}{=} B \qquad f(x) \xrightarrow[x \to \infty]{\text{a.s.}} L \qquad \underset{\theta \in \Theta}{\operatorname{arg\,max}} \; \mathcal{L}(\theta)
$$
```

$$
A \overset{\text{def}}{=} B \qquad f(x) \xrightarrow[x \to \infty]{\text{a.s.}} L \qquad \underset{\theta \in \Theta}{\operatorname{arg\,max}} \; \mathcal{L}(\theta)
$$

---

## 10. Spacing, Formatting & Modulo Arithmetic

### 10.1 Explicit Spacing Commands

| Command | Width | Visual Effect |
| :--- | :--- | :--- |
| `\!` | $-3/18\text{ em}$ | Negative thin space (tightens) |
| `\,` | $3/18\text{ em}$ | Thin space (differentials: `dx\,dy`) |
| `\:` | $4/18\text{ em}$ | Medium space |
| `\;` | $5/18\text{ em}$ | Thick space |
| `\ ` | standard space | Inter-word space |
| `\quad` | $1\text{ em}$ | Quad space |
| `\qquad` | $2\text{ em}$ | Double quad space |

```latex
$$
\int_a^b f(x)\!dx \quad \int_a^b f(x)\,dx \quad a\:b \quad a\;b \quad a\ b \quad a\quad b \quad a\qquad b
$$
```

$$
\int_a^b f(x)\!dx \quad \int_a^b f(x)\,dx \quad a\:b \quad a\;b \quad a\ b \quad a\quad b \quad a\qquad b
$$

### 10.2 Preserving Text Inside Formulas

Always wrap prose inside `\text{...}` so spaces and font ligatures remain upright and legible:

```latex
$$
T(n) = \mathcal{O}(n \log n) \quad \text{for all } n \ge n_0
$$
```

$$
T(n) = \mathcal{O}(n \log n) \quad \text{for all } n \ge n_0
$$

### 10.3 Modulo Formats

```latex
$$
a \equiv b \pmod{m} \qquad a \equiv b \pod{m} \qquad r = a \bmod m
$$
```

$$
a \equiv b \pmod{m} \qquad a \equiv b \pod{m} \qquad r = a \bmod m
$$

---

## 11. Styling, Box Framing & Color Highlights

### 11.1 Framing Results (`\boxed`)

```latex
$$
\boxed{e^{i\pi} + 1 = 0}
$$
```

$$
\boxed{e^{i\pi} + 1 = 0}
$$

### 11.2 Strikeout & Cancellations (`\cancel`, `\xcancel`)

```latex
$$
\frac{\cancel{(x - 1)}(x + 1)}{\cancel{x - 1}} = x + 1 \qquad \lim_{x \to 0} \frac{x^2 + \cancelto{0}{5x^3}}{x^2} = 1
$$
```

$$
\frac{\cancel{(x - 1)}(x + 1)}{\cancel{x - 1}} = x + 1 \qquad \lim_{x \to 0} \frac{x^2 + \cancelto{0}{5x^3}}{x^2} = 1
$$

### 11.3 Colors & Text Highlights

```latex
$$
\color{#2563eb}{\mathbf{F}} = \color{#dc2626}{m}\color{#16a34a}{\mathbf{a}} \qquad \color{#9333ea}{\oint_{\partial \Omega} \omega = \int_{\Omega} d\omega}
$$
```

$$
\color{#2563eb}{\mathbf{F}} = \color{#dc2626}{m}\color{#16a34a}{\mathbf{a}} \qquad \color{#9333ea}{\oint_{\partial \Omega} \omega = \int_{\Omega} d\omega}
$$

---

## 12. Master Showcase: Real-World STEM Formulations

### 12.1 Electromagnetism (Maxwell's Equations)

```latex
$$
\begin{aligned}
\nabla \cdot \mathbf{E} &= \frac{\rho}{\varepsilon_0} && \text{(Gauss's Law)} \\
\nabla \cdot \mathbf{B} &= 0 && \text{(Gauss's Law for Magnetism)} \\
\nabla \times \mathbf{E} &= -\frac{\partial \mathbf{B}}{\partial t} && \text{(Faraday's Law)} \\
\nabla \times \mathbf{B} &= \mu_0 \mathbf{J} + \mu_0 \varepsilon_0 \frac{\partial \mathbf{E}}{\partial t} && \text{(Amp\grave{e}re-Maxwell Law)}
\end{aligned}
$$
```

$$
\begin{aligned}
\nabla \cdot \mathbf{E} &= \frac{\rho}{\varepsilon_0} && \text{(Gauss's Law)} \\
\nabla \cdot \mathbf{B} &= 0 && \text{(Gauss's Law for Magnetism)} \\
\nabla \times \mathbf{E} &= -\frac{\partial \mathbf{B}}{\partial t} && \text{(Faraday's Law)} \\
\nabla \times \mathbf{B} &= \mu_0 \mathbf{J} + \mu_0 \varepsilon_0 \frac{\partial \mathbf{E}}{\partial t} && \text{(Amp\grave{e}re-Maxwell Law)}
\end{aligned}
$$

### 12.2 Quantum Mechanics (Time-Dependent Schrödinger Equation)

```latex
$$
i\hbar \frac{\partial}{\partial t} \Psi(\mathbf{r}, t) = \left[ -\frac{\hbar^2}{2m}\nabla^2 + V(\mathbf{r}, t) \right] \Psi(\mathbf{r}, t)
$$
```

$$
i\hbar \frac{\partial}{\partial t} \Psi(\mathbf{r}, t) = \left[ -\frac{\hbar^2}{2m}\nabla^2 + V(\mathbf{r}, t) \right] \Psi(\mathbf{r}, t)
$$

### 12.3 General Relativity (Einstein Field Equations)

```latex
$$
G_{\mu\nu} + \Lambda g_{\mu\nu} = \frac{8\pi G}{c^4} T_{\mu\nu}
$$
```

$$
G_{\mu\nu} + \Lambda g_{\mu\nu} = \frac{8\pi G}{c^4} T_{\mu\nu}
$$

### 12.4 Machine Learning: Transformer Scaled Dot-Product Attention

```latex
$$
\operatorname{Attention}(\mathbf{Q}, \mathbf{K}, \mathbf{V}) = \operatorname{softmax}\left( \frac{\mathbf{Q}\mathbf{K}^\mathsf{T}}{\sqrt{d_k}} \right) \mathbf{V}
$$
```

$$
\operatorname{Attention}(\mathbf{Q}, \mathbf{K}, \mathbf{V}) = \operatorname{softmax}\left( \frac{\mathbf{Q}\mathbf{K}^\mathsf{T}}{\sqrt{d_k}} \right) \mathbf{V}
$$

### 12.5 Multivariate Gaussian Probability Density

```latex
$$
p(\mathbf{x} \mid \boldsymbol{\mu}, \mathbf{\Sigma}) = \frac{1}{(2\pi)^{D/2} |\mathbf{\Sigma}|^{1/2}} \exp\left( -\frac{1}{2}(\mathbf{x} - \boldsymbol{\mu})^\mathsf{T} \mathbf{\Sigma}^{-1} (\mathbf{x} - \boldsymbol{\mu}) \right)
$$
```

$$
p(\mathbf{x} \mid \boldsymbol{\mu}, \mathbf{\Sigma}) = \frac{1}{(2\pi)^{D/2} |\mathbf{\Sigma}|^{1/2}} \exp\left( -\frac{1}{2}(\mathbf{x} - \boldsymbol{\mu})^\mathsf{T} \mathbf{\Sigma}^{-1} (\mathbf{x} - \boldsymbol{\mu}) \right)
$$

---

## 13. Summary Quick Reference Card

| Need | Syntax Example |
| :--- | :--- |
| **Fractions** | `\frac{a}{b}`, `\dfrac{a}{b}`, `\cfrac{a}{b}` |
| **Powers / Indices** | `x^2`, `x_i`, `{x_i}^2`, `{}_n C_k` |
| **Roots** | `\sqrt{x}`, `\sqrt[3]{x}` |
| **Sum / Product / Integral** | `\sum_{i=1}^n`, `\prod_{i=1}^n`, `\int_a^b f(x)\,dx`, `\oint_C` |
| **Matrices** | `\begin{pmatrix} a & b \\ c & d \end{pmatrix}`, `\begin{bmatrix} ... \end{bmatrix}` |
| **Multi-line Alignment** | `\begin{aligned} a &= b \\ &= c \end{aligned}` |
| **Conditional Cases** | `f(x) = \begin{cases} a & \text{if } x>0 \\ b & \text{otherwise} \end{cases}` |
| **Scaling Brackets** | `\left( ... \right)`, `\left[ ... \right]`, `\left\{ ... \right\}`, `\left. ... \right|` |
| **Font Types** | `\mathbb{R}`, `\mathcal{F}`, `\mathbf{x}`, `\mathrm{d}x`, `\mathfrak{g}` |
| **Framing** | `\boxed{E = mc^2}` |
