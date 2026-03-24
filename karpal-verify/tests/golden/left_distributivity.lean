namespace KarpalVerify

-- property: distributive
-- origin: karpal-algebra::Semiring for i32 [distributive]
theorem left_distributivity (a : Int) (b : Int) (c : Int) : (mul a (add b c)) = (add (mul a b) (mul a c)) := by
  sorry

end KarpalVerify