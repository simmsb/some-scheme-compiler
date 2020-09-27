(define go (lambda (i)
            (let ((ii (- i 1)))
             (if ii
                (let ()
                  (display ii)
                  (go ii))))))

(go 10)

(define make-box (lambda (initial)
                  (cons (lambda () initial)
                        (lambda (v) (set! initial v)))))

(let ((box-fns (make-box 1)))
  (let ((get (car box-fns)))
   (let ((set (cdr box-fns)))
    (display (get))
    (set 23)
    (display (get)))))

(display (^ 1 2))
(display (string-concat "a" "b"))
