## TODO
* If we know there is 1 op, remove possibility from all positions

### 8-Char 2-Op Combos
--------
12+34=56
49+50=99
10+10=20
50+50=100 X
1+1=2     X
1+99=100  X

--------
99-10=89
20-10=10
100-1=99
102-2=100 X

--------
8*90=720
90*8=720
99*9=891
10*1=100
10*10=100 X

--------
100/2=50
891/9=99
100/10=10 X

### 8-Char 3-Op Combos
--------
4+3+3=10
9+9-9=10
2*2*3=12
2*2+6=10
1/1+9=10
9+8/2=13
1-9+10=2
10-9+1=2
1+10-9=2

### Losses
Answer: 51+30=81
Answer: 32+10=42
Answer: 10+62=72
Answer: 35-15=20
Answer: 77-17=60
Answer: 11+40=51
Answer: 36+50=86
Answer: 40+41=81
Answer: 26+30=56
Answer: 89-19=70
Answer: 25+30=55
Answer: 35+30=65

### Bug to Fix
* Bug is mainly fixed I think
* But the failure mode should be cleaner, so it can keep running.

```
=== Playing game 4224 / 1000000
Answer: 512/8=64
Constraint: 0-2 Operator(s), () and not (), a: ExpressionNumberConstraint "a has up to 4 digits range 1..=9999 regex /(?-u)^[5403176289]?[4380157926]?[4986302157]?[1527809436]?$/": range=1..=9999, b: ExpressionNumberConstraint "b has up to 4 digits range 1..=9999 regex /(?-u)^[0123456789]{1,4}$/": range=1..=9999, b2: ExpressionNumberConstraint "b2 has up to 4 digits range 1..=9999 regex /(?-u)^[0123456789]{1,4}$/": range=1..=9999, c: ExpressionNumberConstraint "c has up to 4 digits range 0..=9999 regex /(?-u)^[0123456789]{1,4}$/": range=0..=9999
Turn 1  Guess: 7*8/28=2
Turn 1 Result: --YGY-Y-
7*8/28=2
Equal sign not at 6
Position 0 could be + - / 0 1 2 3 4 5 6 8 9 =
Position 1 could be + - / 0 1 2 3 4 5 6 8 9 =
Position 2 could be + - / 0 1 2 3 4 5 6 9 =
Position 3 is /
Position 4 could be + - / 0 1 3 4 5 6 8 9 =
Position 5 could be + - / 0 1 2 3 4 5 6 9 =
Position 6 could be + - / 0 1 2 3 4 5 6 8 9
Position 7 could be + - / 0 1 3 4 5 6 8 9 =
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Syntax(
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
regex parse error:
    (?-u)^[360529418]?[401836592]?[90241536]?[]?$
                                             ^^
error: unclosed character class
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
)', src/nerdsolver.rs:492:28
```
