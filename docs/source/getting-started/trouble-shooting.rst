Troubleshooting
===============

If you've hit an error and this information doesn't help, feel free to reach out on Slack, or at contact@opendp.org.

Inferred type is XXX, expected YYY
----------------------------------

By the time you invoke a function or call a relation, OpenDP has already determined the type of the data you are supposed to pass.
To ensure type safety, OpenDP checks that the types of the arguments you pass are similar to the type the function is expecting.

Fixing this may be as simple as passing integers instead of floats, or it may require additional preprocessing.
One common cause of this error is passing a scalar ``epsilon`` to a relation that expects an ``(epsilon, delta)`` tuple.