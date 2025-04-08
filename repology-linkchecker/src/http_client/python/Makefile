all:
	flake8 --ignore=D,Q,E501 http_client.py
	mypy http_client.py
	black http_client.py
	isort http_client.py

test:
	echo '{"request_id":1,"url":"https://repology.org/","use_get":false,"address":"185.185.68.145","timeout":60.0}' | python http_client.py
