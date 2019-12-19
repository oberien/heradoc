# Theorems

{definition}
> A natural number `$ n` is called prime if it has exactly two divisors.

{lemma, title="Euclid"}
> If `$ p` is prime and divides `$ ab` then it divides `$ a` or `$ b`.

{corollary, title="Prime factorization"}
> Each number has a canonical decomposition into prime factors.

{theorem, title="The main theorem"}
> There is an infinite number of prime numbers.

{proof}
> Assume `$ p` to be the largest prime number. Define
> ```$$
> P' = 1+ \prod^p_{i=0, i prime} i
> ```
> Then `$ P'` is not divisible by any prime. But it has at least two divisors, one and itself, so there must a prime divisor larger than `$ p`. This is a contradiction.
