; obligation: group_left_inverse
; property: left inverse
; origin: karpal-algebra::Group for i32 [left inverse]
(set-logic ALL)
(declare-const a Int)
(declare-const e Int)
; ask the solver for a counterexample to the law
(assert (not (= (combine (inv a) a) e)))
(check-sat)
(get-model)