'''
For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.examples``.    
'''

def _http_get(url: str) -> str:
    '''
    Normally would use the requests library for this, but we want to avoid extra dependencies.
    '''
    from urllib.parse import urlparse
    import http.client
    parsed = urlparse(url)
    conn = http.client.HTTPSConnection(parsed.hostname, port=parsed.port)
    conn.request("GET", parsed.path)
    response = conn.getresponse()
    assert response.status == 200
    body = response.read().decode()
    conn.close()
    return body

_california_pums = None

def get_california_pums() -> str:
    global _california_pums
    if _california_pums is None:
        _california_pums = _http_get('https://raw.githubusercontent.com/opendp/opendp/main/docs/source/data/PUMS_california_demographics_1000/data.csv')
    return _california_pums