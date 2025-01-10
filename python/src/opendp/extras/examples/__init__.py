'''
For convenience, all the functions of this module are also available from :py:mod:`opendp.prelude`.
We suggest importing under the conventional name ``dp``:

.. code:: python

    >>> import opendp.prelude as dp

The methods of this module will then be accessible at ``dp.examples``.    
'''

from pathlib import Path


def _http_get(url: str) -> bytes:
    '''
    Normally would use the requests library for this, but we want to avoid extra dependencies.

    This only does what we need. Does not handle:
    - plain http
    - redirects
    - url queries
    - explicit port numbers
    '''
    from urllib.parse import urlparse
    import http.client
    parsed = urlparse(url)
    assert parsed.hostname is not None
    conn = http.client.HTTPSConnection(parsed.hostname)
    conn.request("GET", parsed.path)
    response = conn.getresponse()

    assert response.status == 200, f"{response.status} {response.reason}\n{response.headers}"
    body = response.read()
    conn.close()
    return body


def get_california_pums_path() -> Path:
    '''
    Returns the path to a CSV derived from a
    PUMS (Public Use Microdata Sample) file from the US Census.
    A header row is not included. The columns are:
    
    * age
    * sex
    * educ
    * race
    * income
    * married
    '''
    path = Path(__file__).parent / 'california_pums.csv'
    if not path.exists():
        url = 'https://raw.githubusercontent.com/opendp/opendp/main/docs/source/data/PUMS_california_demographics_1000/data.csv'
        path.write_text(_http_get(url).decode())
    return path


def get_france_lfs_path() -> Path:
    '''
    Returns the path to a CSV derived from the
    `EU Labor Force Survey <https://ec.europa.eu/eurostat/web/microdata/public-microdata/labour-force-survey>`_. First row contains the column names.

    Code developed to work with a public microdata set like this could also be used with the scientific use files, and we believe that differential privacy would be a good tool to ensure that statistics derived from scientific use files could not inadvertantly reveal personal information.

    To reduce the download size for the tutorial, we've `preprocessed <https://github.com/opendp/dp-test-datasets/blob/main/data/eurostat/README.ipynb>`_ the data by selecting a single country (France), dropping unused columns, sampling a subset of the rows, and concatenating the result into a single CSV. The code we'll present in the tutorials could be run on the original public microdata, or for that matter, the full private scientific use files.
    '''
    from io import BytesIO
    from zipfile import ZipFile
    path = Path(__file__).parent / 'france_lfs.csv'
    if not path.exists():
        url = 'https://raw.githubusercontent.com/opendp/dp-test-datasets/refs/heads/main/data/sample_FR_LFS.csv.zip'
        france_lfs_bytes = _http_get(url)
        with ZipFile(BytesIO(france_lfs_bytes)) as data_zip:
            path.write_text(data_zip.open('sample_FR_LFS.csv').read().decode())
    return path