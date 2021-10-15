import torch
import torch.nn as nn
from opendp.network.layers.bahdanau import DPBahdanauAttentionScale


class SentenceModule(nn.Module):

    def __init__(self, vocab_size, embedding_size, hidden_size, tagset_size, num_layers=1, bahdanau=False):
        super().__init__()
        # size of input vocabulary
        self.vocab_size = vocab_size
        # size of initial linear layer outputs
        self.embedding_size = embedding_size
        # number of layers internal to the lstm
        self.num_layers = num_layers
        # size of lstm outputs
        self.hidden_size = hidden_size
        # size of target
        self.tagset_size = tagset_size

        self.embedding = nn.Embedding(vocab_size, embedding_size)
        self.lstm = nn.LSTM(embedding_size, hidden_size, num_layers=num_layers)
        self.bahdanau = (DPBahdanauAttentionScale if bahdanau else nn.Identity)(hidden_size, normalize=True)
        self.hidden2tag = nn.Linear(hidden_size, tagset_size)

        self.loss_function = torch.nn.CrossEntropyLoss(reduction='sum')

    def forward(self, x):
        """

        :param x: x.shape == [batch_size, sequence_length]
        :return:
        """
        # x.shape == [batch_size, sequence_length]
        x = torch.atleast_2d(x)

        # x.shape == [batch_size, sequence_length, embedding_size]
        x = self.embedding(x)
        hidden = self._init_hidden(batch_size=x.shape[0])

        # x.shape == [sequence_length, batch_size, features_length]
        x = torch.transpose(x, 0, 1)

        # x.shape == [sequence_length, batch_size, hidden_size]
        x, hidden = self.lstm(x, hidden)

        # x.shape == [batch_size, sequence_length, hidden_size]
        x = torch.transpose(x, 0, 1)

        # conditionally apply bahdanau scaler
        # x.shape == [batch_size, sequence_length, hidden_size]
        x = self.bahdanau(x)

        # linear transform on each step of each sequence in each batch
        # x.shape == [batch_size, sequence_length, tagset_size]
        x = self.hidden2tag(x)

        return x

    def loss(self, x, y):
        """
        Compute loss for all sequence elements over all batches
        :param x: x.shape == [batch_size, sequence_length]
        :param y: y.shape == [batch_size, sequence_length]
        :return:
        """
        # y_pred.shape == [batch_size, sequence_length, tagset_size]
        y_pred = self(x)

        # y_pred.shape == [batch_size * sequence_length, tagset_size]
        y_pred = y_pred.view(-1, self.tagset_size)

        # y_pred.shape == [batch_size * sequence_length]
        y = y.view(-1)

        return self.loss_function(y_pred, y)

    def _init_hidden(self, batch_size):
        # the dimension semantics are [num_layers, batch_size, hidden_size]
        return (torch.rand(self.num_layers, batch_size, self.hidden_size),
                torch.rand(self.num_layers, batch_size, self.hidden_size))


