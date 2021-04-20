import os

import numpy as np
import torch
import torch.nn as nn

from opendp.network import PrivacyAccountant
from opendp.network.layers.bahdanau import DPBahdanauAttentionScale

from pums.download import datasets
from pums.coordinator import ModelCoordinator
from network_pums import main, printf


class LstmModule(nn.Module):

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


def run_lstm_worker(rank, size, epoch_limit=None, private_step_limit=None, federation_scheme='shuffle', queue=None, model_filepath=None, end_event=None):
    """
    Perform federated learning in a ring structure
    :param rank: index for specific data set
    :param size: total ring size
    :param queue: stores values and privacy accountant usage
    :param model_filepath: indicating where to save the model checkpoint
    :return:
    """

    # Every node gets same data for now
    EMBEDDING_SIZE = 6
    HIDDEN_SIZE = 7

    training_data = [
        ("The dog ate the apple".split(), ["DET", "NN", "V", "DET", "NN"]),
        ("Everybody read that book".split(), ["NN", "V", "DET", "NN"])
    ]

    idx_to_tag = ['DET', 'NN', 'V']
    tag_to_idx = {'DET': 0, 'NN': 1, 'V': 2}

    word_to_idx = {}
    for sent, tags in training_data:
        for word in sent:
            if word not in word_to_idx:
                word_to_idx[word] = len(word_to_idx)

    def prepare_sequence(seq, to_idx):
        """Convert sentence/sequence to torch Tensors"""
        idxs = [to_idx[w] for w in seq]
        return torch.LongTensor(idxs)

    model = LstmModule(
        vocab_size=len(word_to_idx),
        embedding_size=EMBEDDING_SIZE,
        hidden_size=HIDDEN_SIZE,
        tagset_size=len(tag_to_idx),
        bahdanau=False)

    accountant = PrivacyAccountant(model, step_epsilon=1.0)
    coordinator = ModelCoordinator(model, rank, size, federation_scheme, end_event=end_event)

    optimizer = torch.optim.SGD(model.parameters(), lr=0.1)

    for epoch in range(epoch_limit):
        coordinator.recv()

        for sentence, tags in training_data:
            # sentence1 = prepare_sequence(training_data[0][0][:4], word_to_idx)
            # sentence2 = prepare_sequence(training_data[1][0][:4], word_to_idx)
            # sentence = torch.stack([
            #     sentence1,
            #     sentence2
            # ])
            # print(sentence.shape)
            #
            # target1 = prepare_sequence(training_data[0][1][:4], tag_to_idx)
            # target2 = prepare_sequence(training_data[1][1][:4], tag_to_idx)
            # target = torch.stack([
            #     target1,
            #     target2
            # ])
            # print(target.shape)
            sentence = prepare_sequence(sentence, word_to_idx)
            target = prepare_sequence(tags, tag_to_idx)

            loss = accountant.model.loss(sentence, target)
            loss.backward()

            accountant.privatize_grad(reduction='sum')

            optimizer.step()
            optimizer.zero_grad()
        accountant.increment_epoch()

        inputs = prepare_sequence(training_data[0][0], word_to_idx)
        tag_scores = model(inputs).detach().numpy()

        tag = [idx_to_tag[idx] for idx in np.argmax(tag_scores[0], axis=1)]
        printf(f"{rank: 4d} | {epoch: 5d} | {tag}     | {training_data[0][1]}", force=True)

        # Ensure that send() does not happen on the last epoch of the last node,
        # since this would send back to the first node (which is done) and hang
        if rank == size - 1 and epoch == epoch_limit - 1:
            # Only save if filename is given
            if model_filepath:
                torch.save({
                    'epoch': epoch,
                    'model_state_dict': model.state_dict(),
                    'optimizer_state_dict': optimizer.state_dict(),
                    'loss': loss
                }, model_filepath)
        else:
            coordinator.send()

    if queue:
        queue.put((tuple(datasets[rank].values()), accountant.compute_usage()))


if __name__ == "__main__":
    model_path = os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', 'model_checkpoints')
    if not os.path.exists(model_path):
        os.mkdir(model_path)

    print("Rank | Epoch | Predicted | Actual")
    main(worker=run_lstm_worker,
         epoch_limit=1000,
         model_filepath=os.path.join(model_path, 'model.pt'),
         federation_scheme='shuffle')
