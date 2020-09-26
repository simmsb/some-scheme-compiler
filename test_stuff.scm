(define go (lambda (i)
            (let ((ii (- i 1)))
             (if ii
                (let ()
                  (println ii)
                  (go ii)
                 )
              ))
            ))

(go 10)

(define make-box (lambda (initial)
                  (cons (lambda () initial)
                        (lambda (v) (set! initial v)))))

(let ((box-fns (make-box 1)))
  (let ((get (car box-fns)))
   (let ((set (cdr box-fns)))
    (println (get))
    (set 23)
    (println (get)))))

(println "hi")
(println (tostring 1))

(println (^ 1 2))
(println (string-concat "a" "b"))
