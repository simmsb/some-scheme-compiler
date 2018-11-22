((lambda (true_ false_ if_)
   (println "hello world")
   (println (+ 4 2))

   (if_ true_
     (lambda () (println "was true"))
     (lambda () (println "was false")))
   
   (if_ false_
     (lambda () (println "was true"))
     (lambda () (println "was false")))

   (((lambda (b)
	   ((lambda (f)
		  (b (lambda (x) ((f f) x))))
		(lambda (f)
		  (b (lambda (x) ((f f) x))))))
	 (lambda (f)
       (lambda (n)
         (println n)
         (f (+ n 1)))))
    0))
   (lambda (a b) a)
   (lambda (a b) b)
   (lambda (c tval fval) ((c tval fval)))
   )
