(if 1
  (display "good")
  (display "bad"))

(if (- 1 1)
 (display "bad")
 (display "good"))

(if ((lambda () 1))
 (display "good")
 (display "bad"))

(if 0
 (display "bad"))

(if 1
 (display "good"))
